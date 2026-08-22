//! Low-overhead stage metrics and an opt-in tracking allocator.

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout};
use std::cell::Cell;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const METRICS_SCHEMA_VERSION: u32 = 1;
const STAGE_COUNT: usize = 18;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Stage {
    SourceLoading,
    ModuleDiscovery,
    Lexing,
    Parsing,
    InterfaceExtraction,
    CacheLookup,
    IncrementalRebuild,
    Hir,
    Mir,
    Linking,
    Bytecode,
    Verification,
    Vm,
    Cache,
    Jit,
    Sapi,
    PreparedExecution,
    Other,
}

impl Stage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceLoading => "source_loading",
            Self::ModuleDiscovery => "module_discovery",
            Self::Lexing => "lexing",
            Self::Parsing => "parsing",
            Self::InterfaceExtraction => "interface_extraction",
            Self::CacheLookup => "cache_lookup",
            Self::IncrementalRebuild => "incremental_rebuild",
            Self::Hir => "hir",
            Self::Mir => "mir",
            Self::Linking => "linking",
            Self::Bytecode => "bytecode",
            Self::Verification => "verification",
            Self::Vm => "vm",
            Self::Cache => "cache",
            Self::Jit => "jit",
            Self::Sapi => "sapi",
            Self::PreparedExecution => "prepared_execution",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct AllocationSnapshot {
    allocations: u64,
    deallocations: u64,
    allocated_bytes: u64,
    deallocated_bytes: u64,
    live_bytes: u64,
    peak_live_bytes: u64,
}

static ALLOCATIONS: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static DEALLOCATIONS: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static ALLOCATED_BYTES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static DEALLOCATED_BYTES: [AtomicU64; STAGE_COUNT] = [const { AtomicU64::new(0) }; STAGE_COUNT];
static LIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static PEAK_LIVE_BYTES: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static CURRENT_STAGE: Cell<Stage> = const { Cell::new(Stage::Other) };
}

fn snapshot(stage: Stage) -> AllocationSnapshot {
    let index = stage as usize;
    AllocationSnapshot {
        allocations: ALLOCATIONS[index].load(Ordering::Relaxed),
        deallocations: DEALLOCATIONS[index].load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES[index].load(Ordering::Relaxed),
        deallocated_bytes: DEALLOCATED_BYTES[index].load(Ordering::Relaxed),
        live_bytes: LIVE_BYTES.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE_BYTES.load(Ordering::Relaxed),
    }
}

fn record_allocation(size: usize) {
    let size = size as u64;
    CURRENT_STAGE.with(|stage| {
        let index = stage.get() as usize;
        ALLOCATIONS[index].fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES[index].fetch_add(size, Ordering::Relaxed);
    });
    let live = LIVE_BYTES.fetch_add(size, Ordering::Relaxed) + size;
    PEAK_LIVE_BYTES.fetch_max(live, Ordering::Relaxed);
}

fn record_deallocation(size: usize) {
    let size = size as u64;
    CURRENT_STAGE.with(|stage| {
        let index = stage.get() as usize;
        DEALLOCATIONS[index].fetch_add(1, Ordering::Relaxed);
        DEALLOCATED_BYTES[index].fetch_add(size, Ordering::Relaxed);
    });
    LIVE_BYTES.fetch_sub(size, Ordering::Relaxed);
}

/// A global allocator wrapper that attributes allocation activity to stages.
///
/// Allocation metadata is not enlarged or moved. The wrapped allocator remains
/// responsible for every pointer and layout.
pub struct TrackingAllocator<A> {
    inner: A,
}

impl<A> TrackingAllocator<A> {
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }
}

// SAFETY: every operation is forwarded to `inner` with the original pointer
// and layout. Metrics update only atomics and non-allocating thread-local cells.
unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller supplies the GlobalAlloc contract to this method,
        // and the exact layout is forwarded to the wrapped allocator.
        let pointer = unsafe { self.inner.alloc(layout) };
        if !pointer.is_null() {
            record_allocation(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record_deallocation(layout.size());
        // SAFETY: `pointer` came from `inner` and is forwarded with the same
        // layout exactly once under the caller's GlobalAlloc contract.
        unsafe { self.inner.dealloc(pointer, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller supplies the GlobalAlloc contract to this method.
        let pointer = unsafe { self.inner.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record_allocation(layout.size());
        }
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: the caller supplies a live allocation from `inner`; all
        // arguments are forwarded unchanged.
        let replacement = unsafe { self.inner.realloc(pointer, layout, new_size) };
        if !replacement.is_null() {
            record_deallocation(layout.size());
            record_allocation(new_size);
        }
        replacement
    }
}

#[derive(Clone, Debug)]
pub struct Measurement {
    pub stage: Stage,
    pub wall: Duration,
    pub allocations: u64,
    pub deallocations: u64,
    pub allocated_bytes: u64,
    pub deallocated_bytes: u64,
    pub retained_bytes_delta: i64,
    pub peak_live_bytes: u64,
    pub items: u64,
    pub output_bytes: u64,
}

impl Measurement {
    pub fn set_output(&mut self, items: usize, bytes: usize) {
        self.items = items as u64;
        self.output_bytes = bytes as u64;
    }
}

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    measurements: Vec<Measurement>,
}

impl Metrics {
    pub fn measure<T>(&mut self, stage: Stage, operation: impl FnOnce() -> T) -> T {
        let previous = CURRENT_STAGE.with(|current| current.replace(stage));
        let before = snapshot(stage);
        let started = Instant::now();
        let result = operation();
        let wall = started.elapsed();
        let after = snapshot(stage);
        CURRENT_STAGE.with(|current| current.set(previous));
        self.measurements.push(Measurement {
            stage,
            wall,
            allocations: after.allocations.saturating_sub(before.allocations),
            deallocations: after.deallocations.saturating_sub(before.deallocations),
            allocated_bytes: after.allocated_bytes.saturating_sub(before.allocated_bytes),
            deallocated_bytes: after
                .deallocated_bytes
                .saturating_sub(before.deallocated_bytes),
            retained_bytes_delta: signed_delta(after.live_bytes, before.live_bytes),
            peak_live_bytes: after.peak_live_bytes,
            items: 0,
            output_bytes: 0,
        });
        result
    }

    pub fn last_mut(&mut self) -> Option<&mut Measurement> {
        self.measurements.last_mut()
    }

    pub fn measurements(&self) -> &[Measurement] {
        &self.measurements
    }

    pub fn to_json(&self) -> String {
        let entries = self
            .measurements
            .iter()
            .map(|measurement| {
                format!(
                    concat!(
                        "{{\"stage\":\"{}\",\"wall_ns\":{},\"allocations\":{},",
                        "\"deallocations\":{},\"allocated_bytes\":{},",
                        "\"deallocated_bytes\":{},\"retained_bytes_delta\":{},",
                        "\"peak_live_bytes\":{},\"items\":{},\"output_bytes\":{}}}"
                    ),
                    measurement.stage.as_str(),
                    measurement.wall.as_nanos(),
                    measurement.allocations,
                    measurement.deallocations,
                    measurement.allocated_bytes,
                    measurement.deallocated_bytes,
                    measurement.retained_bytes_delta,
                    measurement.peak_live_bytes,
                    measurement.items,
                    measurement.output_bytes,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"schema_version\":{METRICS_SCHEMA_VERSION},\"measurements\":[{entries}]}}")
    }
}

fn signed_delta(after: u64, before: u64) -> i64 {
    if after >= before {
        i64::try_from(after - before).unwrap_or(i64::MAX)
    } else {
        -i64::try_from(before - after).unwrap_or(i64::MAX)
    }
}

impl fmt::Display for Metrics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "{:<16} {:>12} {:>12} {:>14} {:>14}",
            "stage", "wall", "allocs", "allocated", "retained"
        )?;
        for measurement in &self.measurements {
            writeln!(
                formatter,
                "{:<16} {:>9.3} ms {:>12} {:>11} B {:>11} B",
                measurement.stage.as_str(),
                measurement.wall.as_secs_f64() * 1_000.0,
                measurement.allocations,
                measurement.allocated_bytes,
                measurement.retained_bytes_delta,
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Metrics, Stage};

    #[test]
    fn records_timing_and_serializes_schema() {
        let mut metrics = Metrics::default();
        let value = metrics.measure(Stage::Parsing, || 42);
        assert_eq!(value, 42);
        assert_eq!(metrics.measurements().len(), 1);
        assert!(metrics.to_json().contains("\"schema_version\":1"));
        assert!(metrics.to_json().contains("\"stage\":\"parsing\""));
    }
}
