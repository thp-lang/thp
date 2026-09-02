//! Request-local THP values and structured runtime failures.

#![allow(unsafe_code)]
#![allow(clippy::match_same_arms)]

use std::alloc::{Layout, alloc};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fmt;
use std::io::{Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use thp_diagnostics::Span;
use thp_hir::{ClassId, PropertyId, Type};

const TAG_NULL: u64 = 0;
const TAG_INT: u64 = 1;
const TAG_FLOAT: u64 = 2;
const TAG_BOOL: u64 = 3;
const TAG_HEAP: u64 = 4;
const PHP_GC_ROOT_THRESHOLD: usize = 10_001;
const PHP_GC_THRESHOLD_STEP: usize = 10_000;

static NEXT_HEAP_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static ACTIVE_HEAP: RefCell<Option<Rc<HeapState>>> = const { RefCell::new(None) };
}

/// Request-scoped managed heap and cycle-collector owner.
///
/// The VM activates one heap while executing a request. Heap values retain the
/// state they were allocated from, so teardown remains safe even when a value
/// temporarily escapes the execution stack.
pub struct RequestHeap {
    state: Rc<HeapState>,
}

impl fmt::Debug for RequestHeap {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestHeap")
            .field("id", &self.state.id)
            .field("stats", &self.stats())
            .finish()
    }
}

impl RequestHeap {
    /// Creates an isolated request heap.
    ///
    /// `None` means no logical heap or handle limit. Collector metadata is
    /// reserved up front so reference-count drops never need to grow it.
    ///
    /// # Errors
    ///
    /// Returns an allocation or heap-limit failure when the collector metadata
    /// cannot be reserved.
    pub fn new(
        max_bytes: Option<usize>,
        max_open_handles: Option<usize>,
    ) -> Result<Self, RuntimeErrorKind> {
        let root_capacity = max_bytes.map_or(PHP_GC_ROOT_THRESHOLD, |limit| {
            PHP_GC_ROOT_THRESHOLD.min((limit / (mem::size_of::<*mut HeapCell>() * 4)).max(16))
        });
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(root_capacity)
            .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
        let metadata_bytes = roots.capacity() * mem::size_of::<*mut HeapCell>();
        if let Some(limit) = max_bytes
            && metadata_bytes > limit
        {
            return Err(RuntimeErrorKind::HeapLimit { limit });
        }
        Ok(Self {
            state: Rc::new(HeapState {
                id: NEXT_HEAP_ID.fetch_add(1, Ordering::Relaxed),
                max_bytes,
                current_bytes: Cell::new(metadata_bytes),
                peak_bytes: Cell::new(metadata_bytes),
                live_cells: Cell::new(0),
                roots: RefCell::new(roots),
                root_threshold: Cell::new(root_capacity),
                collecting: Cell::new(false),
                collections: Cell::new(0),
                collected_cells: Cell::new(0),
                collected_bytes: Cell::new(0),
                max_open_handles,
                open_handles: Cell::new(0),
                peak_open_handles: Cell::new(0),
                #[cfg(test)]
                fail_after: Cell::new(None),
            }),
        })
    }

    /// Makes this heap the allocation owner for the current thread.
    #[must_use]
    pub fn activate(&self) -> ActiveHeapGuard {
        let previous = ACTIVE_HEAP.with(|active| active.replace(Some(Rc::clone(&self.state))));
        ActiveHeapGuard { previous }
    }

    /// Runs a pending or final cycle-collection pass.
    pub fn collect_cycles(&self) -> usize {
        self.state.collect_cycles()
    }

    /// Runs collection after the PHP-style candidate threshold is reached.
    pub fn collect_if_needed(&self) -> usize {
        if self.state.roots.borrow().len() >= self.state.root_threshold.get() {
            self.collect_cycles()
        } else {
            0
        }
    }

    pub fn stats(&self) -> HeapStats {
        self.state.stats()
    }

    #[cfg(test)]
    fn fail_allocations_after(&self, successful_allocations: usize) {
        self.state.fail_after.set(Some(successful_allocations));
    }
}

impl Drop for RequestHeap {
    fn drop(&mut self) {
        self.state.collect_cycles();
    }
}

/// Restores the previously active request heap when a nested execution ends.
pub struct ActiveHeapGuard {
    previous: Option<Rc<HeapState>>,
}

impl Drop for ActiveHeapGuard {
    fn drop(&mut self) {
        ACTIVE_HEAP.with(|active| {
            active.replace(self.previous.take());
        });
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeapStats {
    pub current_bytes: usize,
    pub peak_bytes: usize,
    pub live_cells: usize,
    pub collections: usize,
    pub collected_cells: usize,
    pub collected_bytes: usize,
    pub open_handles: usize,
    pub peak_open_handles: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GcColor {
    Black,
    Purple,
    Grey,
    White,
}

struct HeapState {
    id: u64,
    max_bytes: Option<usize>,
    current_bytes: Cell<usize>,
    peak_bytes: Cell<usize>,
    live_cells: Cell<usize>,
    roots: RefCell<Vec<*mut HeapCell>>,
    root_threshold: Cell<usize>,
    collecting: Cell<bool>,
    collections: Cell<usize>,
    collected_cells: Cell<usize>,
    collected_bytes: Cell<usize>,
    max_open_handles: Option<usize>,
    open_handles: Cell<usize>,
    peak_open_handles: Cell<usize>,
    #[cfg(test)]
    fail_after: Cell<Option<usize>>,
}

impl HeapState {
    fn stats(&self) -> HeapStats {
        HeapStats {
            current_bytes: self.current_bytes.get(),
            peak_bytes: self.peak_bytes.get(),
            live_cells: self.live_cells.get(),
            collections: self.collections.get(),
            collected_cells: self.collected_cells.get(),
            collected_bytes: self.collected_bytes.get(),
            open_handles: self.open_handles.get(),
            peak_open_handles: self.peak_open_handles.get(),
        }
    }

    fn charge(&self, bytes: usize) -> Result<(), RuntimeErrorKind> {
        let next = self
            .current_bytes
            .get()
            .checked_add(bytes)
            .ok_or(RuntimeErrorKind::AllocationFailure)?;
        if self.max_bytes.is_some_and(|limit| next > limit) {
            self.collect_cycles();
            let next = self
                .current_bytes
                .get()
                .checked_add(bytes)
                .ok_or(RuntimeErrorKind::AllocationFailure)?;
            if self.max_bytes.is_some_and(|limit| next > limit) {
                return Err(RuntimeErrorKind::HeapLimit {
                    limit: self.max_bytes.expect("limit exists"),
                });
            }
        }
        #[cfg(test)]
        if let Some(remaining) = self.fail_after.get() {
            if remaining == 0 {
                return Err(RuntimeErrorKind::AllocationFailure);
            }
            self.fail_after.set(Some(remaining - 1));
        }
        self.current_bytes.set(next);
        self.peak_bytes.set(self.peak_bytes.get().max(next));
        Ok(())
    }

    fn release(&self, bytes: usize) {
        self.current_bytes
            .set(self.current_bytes.get().saturating_sub(bytes));
    }

    fn acquire_handle(self: &Rc<Self>) -> Result<HandleLease, RuntimeErrorKind> {
        let next = self.open_handles.get().saturating_add(1);
        if self.max_open_handles.is_some_and(|limit| next > limit) {
            return Err(RuntimeErrorKind::OpenHandleLimit {
                limit: self.max_open_handles.expect("limit exists"),
            });
        }
        self.open_handles.set(next);
        self.peak_open_handles
            .set(self.peak_open_handles.get().max(next));
        Ok(HandleLease {
            owner: Some(Rc::clone(self)),
        })
    }

    fn buffer_candidate(&self, pointer: *mut HeapCell) {
        if self.collecting.get() {
            return;
        }
        // SAFETY: callers hold a live reference to this request-owned cell.
        let cell = unsafe { &*pointer };
        if !cell.data.is_collectable() || cell.buffered.replace(true) {
            return;
        }
        cell.color.set(GcColor::Purple);
        let mut roots = self.roots.borrow_mut();
        if roots.len() < roots.capacity() {
            roots.push(pointer);
        } else {
            cell.buffered.set(false);
            self.root_threshold.set(roots.capacity());
        }
    }

    fn unbuffer(&self, pointer: *mut HeapCell) {
        // This linear removal is confined to the bounded PHP-style root
        // buffer and keeps stale raw pointers out of collection passes.
        let mut roots = self.roots.borrow_mut();
        if let Some(index) = roots.iter().position(|candidate| *candidate == pointer) {
            roots.swap_remove(index);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn collect_cycles(&self) -> usize {
        if self.collecting.replace(true) {
            return 0;
        }
        let mut roots = {
            let mut buffered = self.roots.borrow_mut();
            mem::take(&mut *buffered)
        };
        if roots.is_empty() {
            *self.roots.borrow_mut() = roots;
            self.collecting.set(false);
            return 0;
        }

        let mut work = Vec::with_capacity(roots.len());
        for &root in &roots {
            // SAFETY: buffered cells are removed before ordinary destruction.
            let cell = unsafe { &*root };
            cell.buffered.set(false);
            if cell.color.get() == GcColor::Purple && cell.references.get() > 0 {
                work.push(root);
                while let Some(pointer) = work.pop() {
                    // SAFETY: trial deletion does not free cells.
                    let cell = unsafe { &*pointer };
                    if cell.color.get() != GcColor::Grey {
                        cell.color.set(GcColor::Grey);
                        cell.gc_references.set(i64::from(cell.references.get()));
                    }
                    cell.data.for_each_collectable_child(|child| {
                        let child = child.heap_pointer();
                        // SAFETY: child is retained by the edge being scanned.
                        let child_cell = unsafe { &*child };
                        if child_cell.color.get() != GcColor::Grey {
                            child_cell.color.set(GcColor::Grey);
                            child_cell
                                .gc_references
                                .set(i64::from(child_cell.references.get()));
                            work.push(child);
                        }
                        child_cell
                            .gc_references
                            .set(child_cell.gc_references.get() - 1);
                    });
                }
            } else if cell.color.get() != GcColor::Grey {
                cell.color.set(GcColor::Black);
            }
        }

        for &root in &roots {
            work.push(root);
            while let Some(pointer) = work.pop() {
                // SAFETY: no cells are freed during the scan phase.
                let cell = unsafe { &*pointer };
                if cell.color.get() != GcColor::Grey {
                    continue;
                }
                if cell.gc_references.get() > 0 {
                    let mut black = vec![pointer];
                    while let Some(pointer) = black.pop() {
                        // SAFETY: scan-black only visits retained graph edges.
                        let cell = unsafe { &*pointer };
                        if cell.color.replace(GcColor::Black) == GcColor::Black {
                            continue;
                        }
                        cell.data.for_each_collectable_child(|child| {
                            let child = child.heap_pointer();
                            // SAFETY: child is retained by the scanned edge.
                            let child_cell = unsafe { &*child };
                            child_cell
                                .gc_references
                                .set(child_cell.gc_references.get() + 1);
                            if child_cell.color.get() != GcColor::Black {
                                black.push(child);
                            }
                        });
                    }
                } else {
                    cell.color.set(GcColor::White);
                    cell.data.for_each_collectable_child(|child| {
                        let child = child.heap_pointer();
                        // SAFETY: child is retained by the scanned edge.
                        if unsafe { &*child }.color.get() == GcColor::Grey {
                            work.push(child);
                        }
                    });
                }
            }
        }

        let mut white = Vec::new();
        for &root in &roots {
            work.push(root);
            while let Some(pointer) = work.pop() {
                // SAFETY: white graph cells remain allocated until gathered.
                let cell = unsafe { &*pointer };
                if cell.color.replace(GcColor::Black) != GcColor::White {
                    continue;
                }
                white.push(pointer);
                cell.data.for_each_collectable_child(|child| {
                    let child = child.heap_pointer();
                    // SAFETY: child is retained by the white graph edge.
                    if unsafe { &*child }.color.get() == GcColor::White {
                        work.push(child);
                    }
                });
            }
        }

        let mut bytes = 0usize;
        for &pointer in &white {
            // SAFETY: each white cell is gathered once.
            let cell = unsafe { &*pointer };
            cell.references.set(0);
            bytes = bytes.saturating_add(cell.accounted_bytes.get());
            self.release(cell.accounted_bytes.get());
            self.live_cells.set(self.live_cells.get().saturating_sub(1));
        }
        for &pointer in &white {
            // SAFETY: every gathered cell remains allocated during this
            // phase. Clearing all Value-bearing fields first lets white
            // edges observe the zero reference marker without ever
            // dereferencing a cell that an earlier Box drop already freed.
            unsafe { (*pointer).data.clear_gc_edges() };
        }
        for pointer in white.iter().copied() {
            // SAFETY: trial deletion proved there are no external references.
            // Graph edges were detached while every cell remained allocated,
            // so these Box destructors cannot revisit gathered cells.
            unsafe { drop(Box::from_raw(pointer)) };
        }

        let collected = white.len();
        self.collections.set(self.collections.get() + 1);
        self.collected_cells
            .set(self.collected_cells.get().saturating_add(collected));
        self.collected_bytes
            .set(self.collected_bytes.get().saturating_add(bytes));
        let productive = collected > 0;
        let capacity = roots.capacity();
        let threshold = if productive {
            self.root_threshold
                .get()
                .saturating_sub(PHP_GC_THRESHOLD_STEP)
                .max(capacity.min(PHP_GC_ROOT_THRESHOLD))
        } else {
            self.root_threshold
                .get()
                .saturating_add(PHP_GC_THRESHOLD_STEP)
                .min(capacity)
        };
        self.root_threshold.set(threshold.max(1));
        roots.clear();
        *self.roots.borrow_mut() = roots;
        self.collecting.set(false);
        collected
    }
}

struct HandleLease {
    owner: Option<Rc<HeapState>>,
}

impl Drop for HandleLease {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.take() {
            owner
                .open_handles
                .set(owner.open_handles.get().saturating_sub(1));
        }
    }
}

impl fmt::Debug for HandleLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HandleLease")
            .field("active", &self.owner.is_some())
            .finish()
    }
}

/// A cloneable, request-local body source used by `thp:/input`.
#[derive(Clone)]
pub struct RequestInput {
    state: Rc<RefCell<RequestInputState>>,
}

struct RequestInputState {
    reader: Box<dyn Read>,
    declared_length: Option<u64>,
    max_bytes: Option<u64>,
    max_time: Option<Duration>,
    started: Instant,
    consumed: u64,
    logical_position: u64,
    replay: VecDeque<u8>,
    eof: bool,
    closed: bool,
}

impl fmt::Debug for RequestInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.borrow();
        formatter
            .debug_struct("RequestInput")
            .field("declared_length", &state.declared_length)
            .field("consumed", &state.consumed)
            .field("logical_position", &state.logical_position)
            .field("eof", &state.eof)
            .field("closed", &state.closed)
            .finish()
    }
}

impl RequestInput {
    /// Creates a streaming request body with independent size and time limits.
    ///
    /// # Errors
    ///
    /// Rejects a declared body length that already exceeds the byte limit.
    pub fn new(
        reader: Box<dyn Read>,
        declared_length: Option<u64>,
        max_bytes: Option<u64>,
        max_time: Option<Duration>,
    ) -> Result<Self, RuntimeErrorKind> {
        if let (Some(length), Some(limit)) = (declared_length, max_bytes)
            && length > limit
        {
            return Err(RuntimeErrorKind::InputSizeLimit { limit });
        }
        Ok(Self {
            state: Rc::new(RefCell::new(RequestInputState {
                reader,
                declared_length,
                max_bytes,
                max_time,
                started: Instant::now(),
                consumed: 0,
                logical_position: 0,
                replay: VecDeque::new(),
                eof: false,
                closed: false,
            })),
        })
    }

    /// Creates an unlimited empty request body.
    ///
    /// # Panics
    ///
    /// This constructor has no failing limit or allocation path; a panic would
    /// indicate an internal invariant regression.
    pub fn empty() -> Self {
        Self::new(Box::new(std::io::empty()), Some(0), None, None)
            .expect("an empty unlimited request body is valid")
    }

    /// Creates an in-memory request body.
    ///
    /// # Errors
    ///
    /// Returns an input-size failure when `bytes` exceeds `max_bytes`.
    pub fn from_bytes(
        bytes: Vec<u8>,
        max_bytes: Option<u64>,
        max_time: Option<Duration>,
    ) -> Result<Self, RuntimeErrorKind> {
        let length = bytes.len() as u64;
        Self::new(
            Box::new(std::io::Cursor::new(bytes)),
            Some(length),
            max_bytes,
            max_time,
        )
    }

    pub fn position(&self) -> u64 {
        self.state.borrow().logical_position
    }

    pub fn consumed_bytes(&self) -> u64 {
        self.state.borrow().consumed
    }

    /// Applies additional engine policy to this request body.
    ///
    /// Existing host limits remain in force when they are stricter. This
    /// makes the execution boundary authoritative even when a SAPI created
    /// the input before selecting an engine.
    ///
    /// # Errors
    ///
    /// Rejects a declared or already-consumed body larger than the effective
    /// byte limit.
    pub fn apply_limits(
        &self,
        max_bytes: Option<u64>,
        max_time: Option<Duration>,
    ) -> Result<(), RuntimeErrorKind> {
        let mut state = self.state.borrow_mut();
        state.max_bytes = stricter_limit(state.max_bytes, max_bytes);
        state.max_time = stricter_limit(state.max_time, max_time);
        if let Some(limit) = state.max_bytes
            && (state.declared_length.is_some_and(|length| length > limit)
                || state.consumed > limit)
        {
            return Err(RuntimeErrorKind::InputSizeLimit { limit });
        }
        Ok(())
    }

    fn check_time(state: &RequestInputState) -> Result<(), RuntimeErrorKind> {
        if state
            .max_time
            .is_some_and(|limit| state.started.elapsed() >= limit)
        {
            return Err(RuntimeErrorKind::InputTimeLimit {
                limit: state.max_time.expect("limit exists"),
            });
        }
        Ok(())
    }

    fn read(&self, length: usize) -> Result<Vec<u8>, RuntimeErrorKind> {
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(RuntimeErrorKind::Io("stream is closed".to_owned()));
        }
        Self::check_time(&state)?;
        let length = length.min(64 * 1024);
        let mut output = Vec::new();
        output
            .try_reserve(length.min(64 * 1024))
            .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
        while output.len() < length {
            if let Some(byte) = state.replay.pop_front() {
                output.push(byte);
                continue;
            }
            if state.eof {
                break;
            }
            let remaining = length - output.len();
            let allowed = state.max_bytes.map_or(remaining, |limit| {
                usize::try_from(limit.saturating_sub(state.consumed))
                    .unwrap_or(usize::MAX)
                    .min(remaining)
            });
            let probe = if allowed < remaining {
                allowed.saturating_add(1)
            } else {
                allowed
            };
            if probe == 0 {
                return Err(RuntimeErrorKind::InputSizeLimit {
                    limit: state.max_bytes.expect("zero allowance has a limit"),
                });
            }
            let chunk_length = probe.min(64 * 1024);
            let mut chunk = Vec::new();
            chunk
                .try_reserve_exact(chunk_length)
                .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
            chunk.resize(chunk_length, 0);
            let read = state
                .reader
                .read(&mut chunk)
                .map_err(|error| RuntimeErrorKind::Io(format!("request input failed: {error}")))?;
            Self::check_time(&state)?;
            if read == 0 {
                state.eof = true;
                break;
            }
            state.consumed = state.consumed.saturating_add(read as u64);
            if state.max_bytes.is_some_and(|limit| state.consumed > limit) {
                return Err(RuntimeErrorKind::InputSizeLimit {
                    limit: state.max_bytes.expect("limit exists"),
                });
            }
            output.extend_from_slice(&chunk[..read]);
            if read < chunk.len() {
                break;
            }
        }
        state.logical_position = state.logical_position.saturating_add(output.len() as u64);
        Ok(output)
    }

    fn read_all(&self, limit: Option<usize>) -> Result<Vec<u8>, RuntimeErrorKind> {
        let start = self.position();
        let mut output = Vec::new();
        loop {
            let remaining = limit
                .map_or(64 * 1024, |limit| {
                    limit.saturating_add(1).saturating_sub(output.len())
                })
                .min(64 * 1024);
            if remaining == 0 {
                break;
            }
            let chunk = self.read(remaining)?;
            if chunk.is_empty() {
                break;
            }
            output
                .try_reserve(chunk.len())
                .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
            output.extend_from_slice(&chunk);
            if limit.is_some_and(|limit| output.len() > limit) {
                let mut state = self.state.borrow_mut();
                for byte in output.iter().rev() {
                    state.replay.push_front(*byte);
                }
                state.logical_position = start;
                return Err(RuntimeErrorKind::Io(
                    "stream read limit exceeded".to_owned(),
                ));
            }
        }
        Ok(output)
    }

    fn eof(&self) -> Result<bool, RuntimeErrorKind> {
        {
            let state = self.state.borrow();
            if state.closed {
                return Err(RuntimeErrorKind::Io("stream is closed".to_owned()));
            }
            if !state.replay.is_empty() {
                return Ok(false);
            }
            if state.eof {
                return Ok(true);
            }
        }
        let byte = self.read(1)?;
        if byte.is_empty() {
            return Ok(true);
        }
        let mut state = self.state.borrow_mut();
        state.logical_position = state.logical_position.saturating_sub(1);
        state.replay.push_front(byte[0]);
        Ok(false)
    }

    fn close(&self) {
        let mut state = self.state.borrow_mut();
        state.closed = true;
        state.replay.clear();
    }
}

fn stricter_limit<T: Ord + Copy>(left: Option<T>, right: Option<T>) -> Option<T> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(limit), None) | (None, Some(limit)) => Some(limit),
        (None, None) => None,
    }
}

/// A compact request-local runtime value.
///
/// `Value` is deliberately not `Send` or `Sync`; request heaps and their
/// reference counts are confined to one VM thread.
#[repr(C)]
pub struct Value {
    payload: u64,
    tag: u64,
    not_send_or_sync: PhantomData<Rc<()>>,
}

const _: () = assert!(mem::size_of::<Value>() == 16);

#[allow(clippy::missing_errors_doc)]
impl Value {
    pub const NULL: Self = Self {
        payload: 0,
        tag: TAG_NULL,
        not_send_or_sync: PhantomData,
    };

    pub const fn integer(value: i64) -> Self {
        Self {
            payload: u64::from_ne_bytes(value.to_ne_bytes()),
            tag: TAG_INT,
            not_send_or_sync: PhantomData,
        }
    }

    pub const fn float(value: f64) -> Self {
        Self {
            payload: value.to_bits(),
            tag: TAG_FLOAT,
            not_send_or_sync: PhantomData,
        }
    }

    pub const fn bool(value: bool) -> Self {
        Self {
            payload: value as u64,
            tag: TAG_BOOL,
            not_send_or_sync: PhantomData,
        }
    }

    pub fn try_bytes(value: impl Into<Vec<u8>>) -> Result<Self, RuntimeErrorKind> {
        Self::try_allocate(HeapData::Bytes(value.into()))
    }

    pub fn bytes(value: impl Into<Vec<u8>>) -> Self {
        Self::allocate_unmanaged(HeapData::Bytes(value.into()))
    }

    pub fn try_vector(element_type: Type, values: Vec<Self>) -> Result<Self, RuntimeErrorKind> {
        Self::ensure_values_match_active(values.iter())?;
        Self::try_allocate(HeapData::Vector {
            element_type,
            values,
        })
    }

    pub fn vector(element_type: Type, values: Vec<Self>) -> Self {
        Self::allocate_unmanaged(HeapData::Vector {
            element_type,
            values,
        })
    }

    pub fn try_map(
        key_type: Type,
        value_type: Type,
        entries: Vec<(Self, Self)>,
    ) -> Result<Self, RuntimeErrorKind> {
        Self::ensure_values_match_active(
            entries
                .iter()
                .flat_map(|(key, value)| [key, value].into_iter()),
        )?;
        Self::try_allocate(HeapData::Map {
            key_type,
            value_type,
            entries,
        })
    }

    pub fn map(key_type: Type, value_type: Type, entries: Vec<(Self, Self)>) -> Self {
        Self::allocate_unmanaged(HeapData::Map {
            key_type,
            value_type,
            entries,
        })
    }

    pub fn try_object(class: ClassId, property_count: usize) -> Result<Self, RuntimeErrorKind> {
        let mut properties = Vec::new();
        properties
            .try_reserve_exact(property_count)
            .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
        properties.resize_with(property_count, || None);
        Self::try_allocate(HeapData::Object {
            class,
            properties: RefCell::new(properties),
        })
    }

    pub fn object(class: ClassId, property_count: usize) -> Self {
        Self::allocate_unmanaged(HeapData::Object {
            class,
            properties: RefCell::new(vec![None; property_count]),
        })
    }

    pub fn try_throwable_object(
        class: ClassId,
        property_count: usize,
    ) -> Result<Self, RuntimeErrorKind> {
        let mut properties = Vec::new();
        properties
            .try_reserve_exact(property_count)
            .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
        properties.resize_with(property_count, || None);
        Self::try_allocate(HeapData::Exception {
            class,
            properties: RefCell::new(properties),
            message: RefCell::new(Vec::new()),
            code: Cell::new(0),
            previous: RefCell::new(None),
            target: None,
            system_code: 0,
            suppressed: RefCell::new(Vec::new()),
        })
    }

    pub fn throwable_object(class: ClassId, property_count: usize) -> Self {
        Self::allocate_unmanaged(HeapData::Exception {
            class,
            properties: RefCell::new(vec![None; property_count]),
            message: RefCell::new(Vec::new()),
            code: Cell::new(0),
            previous: RefCell::new(None),
            target: None,
            system_code: 0,
            suppressed: RefCell::new(Vec::new()),
        })
    }

    pub fn try_stream(class: ClassId, bytes: Vec<u8>) -> Result<Self, RuntimeErrorKind> {
        let handle = Self::active_handle_lease()?;
        Self::try_allocate(HeapData::Stream {
            class,
            state: RefCell::new(StreamState {
                storage: StreamStorage::Memory(bytes),
                cursor: 0,
                closed: false,
                spill_threshold: None,
                handle,
            }),
        })
    }

    pub fn stream(class: ClassId, bytes: Vec<u8>) -> Self {
        Self::allocate_unmanaged(HeapData::Stream {
            class,
            state: RefCell::new(StreamState {
                storage: StreamStorage::Memory(bytes),
                cursor: 0,
                closed: false,
                spill_threshold: None,
                handle: None,
            }),
        })
    }

    pub fn try_temp_stream(
        class: ClassId,
        spill_threshold: usize,
    ) -> Result<Self, RuntimeErrorKind> {
        let handle = Self::active_handle_lease()?;
        Self::try_allocate(HeapData::Stream {
            class,
            state: RefCell::new(StreamState {
                storage: StreamStorage::Memory(Vec::new()),
                cursor: 0,
                closed: false,
                spill_threshold: Some(spill_threshold),
                handle,
            }),
        })
    }

    pub fn temp_stream(class: ClassId, spill_threshold: usize) -> Self {
        Self::allocate_unmanaged(HeapData::Stream {
            class,
            state: RefCell::new(StreamState {
                storage: StreamStorage::Memory(Vec::new()),
                cursor: 0,
                closed: false,
                spill_threshold: Some(spill_threshold),
                handle: None,
            }),
        })
    }

    pub fn try_request_input_stream(
        class: ClassId,
        input: RequestInput,
    ) -> Result<Self, RuntimeErrorKind> {
        let handle = Self::active_handle_lease()?;
        Self::try_allocate(HeapData::Stream {
            class,
            state: RefCell::new(StreamState {
                storage: StreamStorage::Input(input),
                cursor: 0,
                closed: false,
                spill_threshold: None,
                handle,
            }),
        })
    }

    pub fn try_exception(
        class: ClassId,
        message: Vec<u8>,
        target: Option<Vec<u8>>,
        system_code: i64,
    ) -> Result<Self, RuntimeErrorKind> {
        Self::try_allocate(HeapData::Exception {
            class,
            properties: RefCell::new(Vec::new()),
            message: RefCell::new(message),
            code: Cell::new(system_code),
            previous: RefCell::new(None),
            target,
            system_code,
            suppressed: RefCell::new(Vec::new()),
        })
    }

    pub fn exception(
        class: ClassId,
        message: Vec<u8>,
        target: Option<Vec<u8>>,
        system_code: i64,
    ) -> Self {
        Self::allocate_unmanaged(HeapData::Exception {
            class,
            properties: RefCell::new(Vec::new()),
            message: RefCell::new(message),
            code: Cell::new(system_code),
            previous: RefCell::new(None),
            target,
            system_code,
            suppressed: RefCell::new(Vec::new()),
        })
    }

    fn allocate_unmanaged(data: HeapData) -> Self {
        let cell = Box::new(HeapCell {
            references: Cell::new(1),
            gc_references: Cell::new(0),
            color: Cell::new(GcColor::Black),
            buffered: Cell::new(false),
            accounted_bytes: Cell::new(0),
            owner: None,
            data,
        });
        Self {
            payload: Box::into_raw(cell) as usize as u64,
            tag: TAG_HEAP,
            not_send_or_sync: PhantomData,
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    fn try_allocate(data: HeapData) -> Result<Self, RuntimeErrorKind> {
        let owner = ACTIVE_HEAP.with(|active| active.borrow().clone());
        let Some(owner) = owner else {
            return Ok(Self::allocate_unmanaged(data));
        };
        let accounted_bytes = mem::size_of::<HeapCell>().saturating_add(data.dynamic_size());
        owner.charge(accounted_bytes)?;
        let layout = Layout::new::<HeapCell>();
        // SAFETY: `layout` exactly describes `HeapCell`; a non-null allocation
        // is initialized before any reference to it is created.
        let pointer = unsafe { alloc(layout).cast::<HeapCell>() };
        if pointer.is_null() {
            owner.release(accounted_bytes);
            owner.collect_cycles();
            // SAFETY: same invariant as the first attempt.
            let pointer = unsafe { alloc(layout).cast::<HeapCell>() };
            if pointer.is_null() {
                return Err(RuntimeErrorKind::AllocationFailure);
            }
            // SAFETY: the allocation is suitably aligned and uniquely owned.
            unsafe {
                pointer.write(HeapCell {
                    references: Cell::new(1),
                    gc_references: Cell::new(0),
                    color: Cell::new(GcColor::Black),
                    buffered: Cell::new(false),
                    accounted_bytes: Cell::new(accounted_bytes),
                    owner: Some(Rc::clone(&owner)),
                    data,
                });
            }
            owner
                .current_bytes
                .set(owner.current_bytes.get().saturating_add(accounted_bytes));
            owner
                .peak_bytes
                .set(owner.peak_bytes.get().max(owner.current_bytes.get()));
            owner.live_cells.set(owner.live_cells.get() + 1);
            return Ok(Self::from_heap_pointer(pointer));
        }
        // SAFETY: the allocation is suitably aligned and uniquely owned.
        unsafe {
            pointer.write(HeapCell {
                references: Cell::new(1),
                gc_references: Cell::new(0),
                color: Cell::new(GcColor::Black),
                buffered: Cell::new(false),
                accounted_bytes: Cell::new(accounted_bytes),
                owner: Some(Rc::clone(&owner)),
                data,
            });
        }
        owner.live_cells.set(owner.live_cells.get() + 1);
        Ok(Self::from_heap_pointer(pointer))
    }

    fn from_heap_pointer(pointer: *mut HeapCell) -> Self {
        Self {
            payload: pointer as usize as u64,
            tag: TAG_HEAP,
            not_send_or_sync: PhantomData,
        }
    }

    fn active_handle_lease() -> Result<Option<HandleLease>, RuntimeErrorKind> {
        ACTIVE_HEAP.with(|active| {
            active
                .borrow()
                .as_ref()
                .map(Rc::clone)
                .map_or(Ok(None), |owner| owner.acquire_handle().map(Some))
        })
    }

    fn ensure_values_match_active<'a>(
        values: impl IntoIterator<Item = &'a Self>,
    ) -> Result<(), RuntimeErrorKind> {
        let active = ACTIVE_HEAP.with(|owner| owner.borrow().as_ref().map(|owner| owner.id));
        for value in values {
            if let Some(owner) = value.owner_id()
                && Some(owner) != active
            {
                return Err(RuntimeErrorKind::CrossRequestValue);
            }
        }
        Ok(())
    }

    pub const fn is_null(&self) -> bool {
        self.tag == TAG_NULL
    }

    pub const fn as_int(&self) -> Option<i64> {
        if self.tag == TAG_INT {
            Some(i64::from_ne_bytes(self.payload.to_ne_bytes()))
        } else {
            None
        }
    }

    pub const fn as_float(&self) -> Option<f64> {
        if self.tag == TAG_FLOAT {
            Some(f64::from_bits(self.payload))
        } else {
            None
        }
    }

    pub const fn as_bool(&self) -> Option<bool> {
        if self.tag == TAG_BOOL {
            Some(self.payload != 0)
        } else {
            None
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self.heap_data()? {
            HeapData::Bytes(bytes) => Some(bytes),
            HeapData::Vector { .. }
            | HeapData::Map { .. }
            | HeapData::Object { .. }
            | HeapData::Stream { .. }
            | HeapData::Exception { .. } => None,
        }
    }

    pub fn vector_values(&self) -> Option<&[Self]> {
        match self.heap_data()? {
            HeapData::Vector { values, .. } => Some(values),
            HeapData::Bytes(_)
            | HeapData::Map { .. }
            | HeapData::Object { .. }
            | HeapData::Stream { .. }
            | HeapData::Exception { .. } => None,
        }
    }

    pub fn map_entries(&self) -> Option<&[(Self, Self)]> {
        match self.heap_data()? {
            HeapData::Map { entries, .. } => Some(entries),
            HeapData::Bytes(_)
            | HeapData::Vector { .. }
            | HeapData::Object { .. }
            | HeapData::Stream { .. }
            | HeapData::Exception { .. } => None,
        }
    }

    /// Appends a value after detaching shared vector storage.
    ///
    /// # Errors
    ///
    /// Returns a type error when this value is not a vector.
    pub fn vector_push(&mut self, value: Self) -> Result<usize, RuntimeErrorKind> {
        self.ensure_same_owner(&value)?;
        self.make_heap_unique()?;
        let needs_growth = matches!(
            self.heap_data(),
            Some(HeapData::Vector { values, .. }) if values.len() == values.capacity()
        );
        let growth = usize::from(needs_growth) * mem::size_of::<Value>();
        self.charge_growth(growth)?;
        match self.heap_data_mut() {
            Some(HeapData::Vector { values, .. }) => {
                if let Err(_error) = values.try_reserve_exact(1) {
                    self.release_growth(growth);
                    return Err(RuntimeErrorKind::AllocationFailure);
                }
                values.push(value);
                Ok(values.len())
            }
            _ => Err(RuntimeErrorKind::TypeError(
                "array_push requires a vector".to_owned(),
            )),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.tag {
            TAG_NULL => "null",
            TAG_INT => "int",
            TAG_FLOAT => "float",
            TAG_BOOL => "bool",
            TAG_HEAP => match &self.heap_cell().data {
                HeapData::Bytes(_) => "string",
                HeapData::Vector { .. } => "vector",
                HeapData::Map { .. } => "map",
                HeapData::Object { .. } => "object",
                HeapData::Stream { .. } => "stream",
                HeapData::Exception { .. } => "exception",
            },
            _ => "invalid",
        }
    }

    pub fn count(&self) -> Option<usize> {
        match self.heap_data()? {
            HeapData::Bytes(bytes) => Some(bytes.len()),
            HeapData::Vector { values, .. } => Some(values.len()),
            HeapData::Map { entries, .. } => Some(entries.len()),
            HeapData::Object { .. } => None,
            HeapData::Stream { .. } => None,
            HeapData::Exception { .. } => None,
        }
    }

    /// Returns the number of elements in a vector or map.
    ///
    /// # Errors
    ///
    /// Returns a type error when this value is not an iterable collection.
    pub fn collection_len(&self) -> Result<usize, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Vector { values, .. }) => Ok(values.len()),
            Some(HeapData::Map { entries, .. }) => Ok(entries.len()),
            _ => Err(RuntimeErrorKind::TypeError(format!(
                "{} is not an iterable collection",
                self.type_name()
            ))),
        }
    }

    /// Returns the key at an insertion-order offset.
    ///
    /// Vector keys are their zero-based integer offsets.
    ///
    /// # Errors
    ///
    /// Returns a type or bounds error for an invalid collection or offset.
    pub fn collection_key_at(&self, offset: usize) -> Result<Self, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Vector { values, .. }) => {
                if offset >= values.len() {
                    return Err(RuntimeErrorKind::Bounds(
                        "collection iteration offset out of bounds".to_owned(),
                    ));
                }
                let offset = i64::try_from(offset).map_err(|_| {
                    RuntimeErrorKind::Bounds(
                        "collection iteration offset exceeds the signed 64-bit range".to_owned(),
                    )
                })?;
                Ok(Self::integer(offset))
            }
            Some(HeapData::Map { entries, .. }) => entries
                .get(offset)
                .map(|(key, _)| key.clone())
                .ok_or_else(|| {
                    RuntimeErrorKind::Bounds("collection iteration offset out of bounds".to_owned())
                }),
            _ => Err(RuntimeErrorKind::TypeError(format!(
                "{} is not an iterable collection",
                self.type_name()
            ))),
        }
    }

    /// Returns the value at an insertion-order offset.
    ///
    /// # Errors
    ///
    /// Returns a type or bounds error for an invalid collection or offset.
    pub fn collection_value_at(&self, offset: usize) -> Result<Self, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Vector { values, .. }) => values.get(offset).cloned().ok_or_else(|| {
                RuntimeErrorKind::Bounds("collection iteration offset out of bounds".to_owned())
            }),
            Some(HeapData::Map { entries, .. }) => entries
                .get(offset)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| {
                    RuntimeErrorKind::Bounds("collection iteration offset out of bounds".to_owned())
                }),
            _ => Err(RuntimeErrorKind::TypeError(format!(
                "{} is not an iterable collection",
                self.type_name()
            ))),
        }
    }

    pub fn class_id(&self) -> Option<ClassId> {
        match self.heap_data()? {
            HeapData::Object { class, .. }
            | HeapData::Stream { class, .. }
            | HeapData::Exception { class, .. } => Some(*class),
            HeapData::Bytes(_) | HeapData::Vector { .. } | HeapData::Map { .. } => None,
        }
    }

    /// Reads an initialized object property.
    ///
    /// # Errors
    ///
    /// Returns a type, bounds, or uninitialized-property error.
    pub fn property(&self, property: PropertyId) -> Result<Self, RuntimeErrorKind> {
        let Some(HeapData::Object { properties, .. } | HeapData::Exception { properties, .. }) =
            self.heap_data()
        else {
            return Err(RuntimeErrorKind::TypeError(
                "property access requires an object".to_owned(),
            ));
        };
        let properties = properties.borrow();
        properties
            .get(property.0 as usize)
            .ok_or(RuntimeErrorKind::Bounds(
                "property index out of bounds".to_owned(),
            ))?
            .clone()
            .ok_or(RuntimeErrorKind::UninitializedProperty(property.0))
    }

    /// Mutates an object property. Object aliases intentionally observe the
    /// same mutation; objects do not use collection copy-on-write semantics.
    ///
    /// # Errors
    ///
    /// Returns a type or bounds error for an invalid receiver/property pair.
    pub fn set_property(&self, property: PropertyId, value: Self) -> Result<(), RuntimeErrorKind> {
        self.ensure_same_owner(&value)?;
        let Some(HeapData::Object { properties, .. } | HeapData::Exception { properties, .. }) =
            self.heap_data()
        else {
            return Err(RuntimeErrorKind::TypeError(
                "property assignment requires an object".to_owned(),
            ));
        };
        let mut properties = properties.borrow_mut();
        let Some(slot) = properties.get_mut(property.0 as usize) else {
            return Err(RuntimeErrorKind::Bounds(
                "property index out of bounds".to_owned(),
            ));
        };
        *slot = Some(value);
        Ok(())
    }

    /// Returns the shared stream cursor.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream or closed stream.
    pub fn stream_tell(&self) -> Result<usize, RuntimeErrorKind> {
        let state = self.stream_state()?;
        let state = state.borrow();
        ensure_stream_open(&state)?;
        Ok(state.cursor)
    }

    /// Reads up to `length` bytes and advances the shared cursor.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream or closed stream.
    pub fn stream_read(&self, length: usize) -> Result<Vec<u8>, RuntimeErrorKind> {
        let state = self.stream_state()?;
        let mut state = state.borrow_mut();
        ensure_stream_open(&state)?;
        let bytes = match &state.storage {
            StreamStorage::Input(input) => input.read(length)?,
            StreamStorage::Closed => {
                return Err(RuntimeErrorKind::Io("stream is closed".to_owned()));
            }
            StreamStorage::Memory(_) | StreamStorage::File { .. } => {
                let start = state.cursor;
                let end = start.saturating_add(length).min(state.len());
                state.read_range(start, end)?
            }
        };
        state.cursor = state.cursor.saturating_add(bytes.len());
        Ok(bytes)
    }

    /// Reads the remaining bytes without advancing when `limit` is exceeded.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream, closed stream, or exceeded limit.
    pub fn stream_read_all(&self, limit: Option<usize>) -> Result<Vec<u8>, RuntimeErrorKind> {
        let state = self.stream_state()?;
        let mut state = state.borrow_mut();
        ensure_stream_open(&state)?;
        let bytes = match &state.storage {
            StreamStorage::Input(input) => input.read_all(limit)?,
            StreamStorage::Closed => {
                return Err(RuntimeErrorKind::Io("stream is closed".to_owned()));
            }
            StreamStorage::Memory(_) | StreamStorage::File { .. } => {
                let remaining = state.len().saturating_sub(state.cursor);
                if limit.is_some_and(|limit| remaining > limit) {
                    return Err(RuntimeErrorKind::Io(
                        "stream read limit exceeded".to_owned(),
                    ));
                }
                let start = state.cursor;
                let end = state.len();
                state.read_range(start, end)?
            }
        };
        state.cursor = state.cursor.saturating_add(bytes.len());
        Ok(bytes)
    }

    /// Reports whether the shared cursor is at the end.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream or closed stream.
    pub fn stream_eof(&self) -> Result<bool, RuntimeErrorKind> {
        let state = self.stream_state()?;
        let state = state.borrow();
        ensure_stream_open(&state)?;
        match &state.storage {
            StreamStorage::Input(input) => input.eof(),
            StreamStorage::Closed => Err(RuntimeErrorKind::Io("stream is closed".to_owned())),
            StreamStorage::Memory(_) | StreamStorage::File { .. } => {
                Ok(state.cursor >= state.len())
            }
        }
    }

    /// Moves the shared cursor to an absolute byte position.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream or closed stream.
    pub fn stream_seek(&self, position: usize) -> Result<(), RuntimeErrorKind> {
        let state = self.stream_state()?;
        let mut state = state.borrow_mut();
        ensure_stream_open(&state)?;
        if matches!(state.storage, StreamStorage::Input(_)) {
            return Err(RuntimeErrorKind::Io(
                "stream does not support seeking".to_owned(),
            ));
        }
        state.cursor = position;
        Ok(())
    }

    /// Writes all bytes at the cursor, zero-filling any gap.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-stream, closed stream, or position overflow.
    pub fn stream_write_all(&self, bytes: &[u8]) -> Result<(), RuntimeErrorKind> {
        let state = self.stream_state()?;
        let mut state = state.borrow_mut();
        ensure_stream_open(&state)?;
        if matches!(state.storage, StreamStorage::Input(_)) {
            return Err(RuntimeErrorKind::Io(
                "stream does not support writing".to_owned(),
            ));
        }
        let end = state
            .cursor
            .checked_add(bytes.len())
            .ok_or_else(|| RuntimeErrorKind::Io("stream position overflow".to_owned()))?;
        let previous_memory_capacity = match &state.storage {
            StreamStorage::Memory(storage) => storage.capacity(),
            _ => 0,
        };
        state.spill_if_needed(end)?;
        let spilled =
            previous_memory_capacity > 0 && matches!(state.storage, StreamStorage::File { .. });
        if spilled {
            self.release_growth(previous_memory_capacity);
        }
        let growth = match &state.storage {
            StreamStorage::Memory(storage) => end.saturating_sub(storage.capacity()),
            _ => 0,
        };
        self.charge_growth(growth)?;
        if let Err(error) = state.write_at_cursor(bytes) {
            self.release_growth(growth);
            return Err(error);
        }
        state.cursor = end;
        Ok(())
    }

    /// Idempotently closes the shared stream state.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a stream.
    pub fn stream_close(&self) -> Result<(), RuntimeErrorKind> {
        let state = self.stream_state()?;
        let mut state = state.borrow_mut();
        if state.closed {
            return Ok(());
        }
        if let StreamStorage::Input(input) = &state.storage {
            input.close();
        }
        let released = match &state.storage {
            StreamStorage::Memory(bytes) => bytes.capacity(),
            _ => 0,
        };
        state.storage = StreamStorage::Closed;
        state.handle.take();
        state.closed = true;
        self.release_growth(released);
        Ok(())
    }

    /// Reports the shared closed state.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a stream.
    pub fn stream_is_closed(&self) -> Result<bool, RuntimeErrorKind> {
        Ok(self.stream_state()?.borrow().closed)
    }

    /// Borrows a native exception message.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a native exception.
    pub fn exception_message(&self) -> Result<Vec<u8>, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception { message, .. }) => Ok(message.borrow().clone()),
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Initializes the common state of a throwable object.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a throwable object.
    pub fn initialize_exception(
        &self,
        message_value: Vec<u8>,
        code_value: i64,
        previous_value: Option<Self>,
    ) -> Result<(), RuntimeErrorKind> {
        if let Some(previous) = &previous_value {
            self.ensure_same_owner(previous)?;
        }
        let old_capacity = match self.heap_data() {
            Some(HeapData::Exception { message, .. }) => message.borrow().capacity(),
            _ => 0,
        };
        let growth = message_value.capacity().saturating_sub(old_capacity);
        self.charge_growth(growth)?;
        match self.heap_data() {
            Some(HeapData::Exception {
                message,
                code,
                previous,
                ..
            }) => {
                *message.borrow_mut() = message_value;
                if old_capacity > message.borrow().capacity() {
                    self.release_growth(old_capacity - message.borrow().capacity());
                }
                code.set(code_value);
                *previous.borrow_mut() = previous_value;
                Ok(())
            }
            _ => Err(RuntimeErrorKind::TypeError(
                "exception constructor requires a throwable object".to_owned(),
            )),
        }
    }

    /// Returns a throwable's application code.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a throwable object.
    pub fn exception_code(&self) -> Result<i64, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception { code, .. }) => Ok(code.get()),
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Clones a throwable's previous value or returns null.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a throwable object.
    pub fn exception_previous(&self) -> Result<Self, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception { previous, .. }) => {
                Ok(previous.borrow().clone().unwrap_or(Self::NULL))
            }
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Appends a throwable to the end of the existing previous chain.
    ///
    /// # Errors
    ///
    /// Returns an error when a chain member is not a throwable or the chain is
    /// cyclic or unreasonably deep.
    pub fn append_previous(&self, pending: Self) -> Result<(), RuntimeErrorKind> {
        self.ensure_same_owner(&pending)?;
        if pending == *self {
            return Ok(());
        }
        let mut pending_member = pending.clone();
        for depth in 0..1024 {
            if pending_member == *self {
                return Ok(());
            }
            let next = match pending_member.heap_data() {
                Some(HeapData::Exception { previous, .. }) => previous.borrow().clone(),
                _ => {
                    return Err(RuntimeErrorKind::TypeError(
                        "previous chaining requires throwable objects".to_owned(),
                    ));
                }
            };
            let Some(next) = next else {
                break;
            };
            if depth == 1023 {
                return Err(RuntimeErrorKind::Bounds(
                    "throwable previous chain exceeds 1024 entries".to_owned(),
                ));
            }
            pending_member = next;
        }
        let mut current = self.clone();
        for _ in 0..1024 {
            let next = match current.heap_data() {
                Some(HeapData::Exception { previous, .. }) => previous.borrow().clone(),
                _ => {
                    return Err(RuntimeErrorKind::TypeError(
                        "previous chaining requires throwable objects".to_owned(),
                    ));
                }
            };
            if let Some(next) = next {
                if next == pending {
                    return Ok(());
                }
                if next == current || next == *self {
                    return Err(RuntimeErrorKind::TypeError(
                        "throwable previous chain is cyclic".to_owned(),
                    ));
                }
                current = next;
                continue;
            }
            let Some(HeapData::Exception { previous, .. }) = current.heap_data() else {
                unreachable!("chain member was checked above")
            };
            *previous.borrow_mut() = Some(pending);
            return Ok(());
        }
        Err(RuntimeErrorKind::Bounds(
            "throwable previous chain exceeds 1024 entries".to_owned(),
        ))
    }

    /// Borrows a native exception target, or an empty byte string.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a native exception.
    pub fn exception_target(&self) -> Result<&[u8], RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception {
                target: Some(target),
                ..
            }) => Ok(target),
            Some(HeapData::Exception { target: None, .. }) => Ok(&[]),
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Returns a native exception's platform error code.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a native exception.
    pub fn exception_system_code(&self) -> Result<i64, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception { system_code, .. }) => Ok(*system_code),
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Clones the suppressed-exception list.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a native exception.
    pub fn exception_suppressed(&self) -> Result<Vec<Self>, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Exception { suppressed, .. }) => Ok(suppressed.borrow().clone()),
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            )),
        }
    }

    /// Attaches a cleanup failure to a primary native exception.
    ///
    /// # Errors
    ///
    /// Returns an error when the receiver is not a native exception.
    pub fn add_suppressed(&self, exception: Self) -> Result<(), RuntimeErrorKind> {
        self.ensure_same_owner(&exception)?;
        let needs_growth = matches!(
            self.heap_data(),
            Some(HeapData::Exception { suppressed, .. })
                if suppressed.borrow().len() == suppressed.borrow().capacity()
        );
        let growth = usize::from(needs_growth) * mem::size_of::<Value>();
        self.charge_growth(growth)?;
        match self.heap_data() {
            Some(HeapData::Exception { suppressed, .. }) => {
                let mut suppressed = suppressed.borrow_mut();
                if let Err(_error) = suppressed.try_reserve_exact(1) {
                    self.release_growth(growth);
                    return Err(RuntimeErrorKind::AllocationFailure);
                }
                suppressed.push(exception);
                Ok(())
            }
            _ => Err(RuntimeErrorKind::TypeError(
                "exception method requires an exception object".to_owned(),
            ))
            .inspect_err(|_| self.release_growth(growth)),
        }
    }

    /// Reads a vector, map, or byte-string element.
    ///
    /// # Errors
    ///
    /// Returns a type or bounds error for an invalid index operation.
    pub fn index(&self, index: &Self) -> Result<Self, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Vector { values, .. }) => {
                let Some(index) = index.as_int() else {
                    return Err(RuntimeErrorKind::TypeError(
                        "vector index must be int".to_owned(),
                    ));
                };
                let index = usize::try_from(index)
                    .map_err(|_| RuntimeErrorKind::Bounds("negative vector index".to_owned()))?;
                values.get(index).cloned().ok_or_else(|| {
                    RuntimeErrorKind::Bounds("vector index out of bounds".to_owned())
                })
            }
            Some(HeapData::Map { entries, .. }) => entries
                .iter()
                .find(|(key, _)| key == index)
                .map(|(_, value)| value.clone())
                .ok_or_else(|| RuntimeErrorKind::Bounds("map key does not exist".to_owned())),
            Some(HeapData::Bytes(bytes)) => {
                let Some(index) = index.as_int() else {
                    return Err(RuntimeErrorKind::TypeError(
                        "string offset must be int".to_owned(),
                    ));
                };
                let index = usize::try_from(index)
                    .map_err(|_| RuntimeErrorKind::Bounds("negative string offset".to_owned()))?;
                bytes
                    .get(index)
                    .map(|byte| Self::bytes(vec![*byte]))
                    .ok_or_else(|| {
                        RuntimeErrorKind::Bounds("string offset out of bounds".to_owned())
                    })
            }
            Some(
                HeapData::Object { .. } | HeapData::Stream { .. } | HeapData::Exception { .. },
            )
            | None => Err(RuntimeErrorKind::TypeError(format!(
                "{} cannot be indexed",
                self.type_name()
            ))),
        }
    }

    /// Replaces a vector element or inserts/replaces a map entry after
    /// detaching shared collection storage.
    ///
    /// # Errors
    ///
    /// Returns a type or bounds error for an invalid collection or index.
    pub fn set_index(&mut self, index: &Self, value: Self) -> Result<(), RuntimeErrorKind> {
        self.ensure_same_owner(index)?;
        self.ensure_same_owner(&value)?;
        self.make_heap_unique()?;
        let map_needs_growth = matches!(
            self.heap_data(),
            Some(HeapData::Map { entries, .. })
                if !entries.iter().any(|(key, _)| key == index)
                    && entries.len() == entries.capacity()
        );
        let map_growth = usize::from(map_needs_growth) * mem::size_of::<(Value, Value)>();
        self.charge_growth(map_growth)?;
        match self.heap_data_mut() {
            Some(HeapData::Vector { values, .. }) => {
                let Some(index) = index.as_int() else {
                    return Err(RuntimeErrorKind::TypeError(
                        "vector index must be int".to_owned(),
                    ));
                };
                let index = usize::try_from(index)
                    .map_err(|_| RuntimeErrorKind::Bounds("negative vector index".to_owned()))?;
                let slot = values.get_mut(index).ok_or_else(|| {
                    RuntimeErrorKind::Bounds("vector index out of bounds".to_owned())
                })?;
                *slot = value;
                Ok(())
            }
            Some(HeapData::Map { entries, .. }) => {
                if let Some((_, slot)) = entries.iter_mut().find(|(key, _)| key == index) {
                    *slot = value;
                } else {
                    if let Err(_error) = entries.try_reserve_exact(1) {
                        self.release_growth(map_growth);
                        return Err(RuntimeErrorKind::AllocationFailure);
                    }
                    entries.push((index.clone(), value));
                }
                Ok(())
            }
            _ => {
                self.release_growth(map_growth);
                Err(RuntimeErrorKind::TypeError(format!(
                    "{} does not support element assignment",
                    self.type_name()
                )))
            }
        }
    }

    pub fn output_bytes(&self) -> Option<Vec<u8>> {
        match self.tag {
            TAG_INT => Some(
                i64::from_ne_bytes(self.payload.to_ne_bytes())
                    .to_string()
                    .into_bytes(),
            ),
            TAG_FLOAT => Some(format_output_float(f64::from_bits(self.payload)).into_bytes()),
            TAG_BOOL if self.as_bool() == Some(true) => Some(b"true".to_vec()),
            TAG_BOOL => Some(b"false".to_vec()),
            TAG_HEAP => match &self.heap_cell().data {
                HeapData::Bytes(bytes) => Some(bytes.clone()),
                HeapData::Vector { .. }
                | HeapData::Map { .. }
                | HeapData::Object { .. }
                | HeapData::Stream { .. }
                | HeapData::Exception { .. } => None,
            },
            _ => None,
        }
    }

    pub fn dump(&self) -> Vec<u8> {
        match self.tag {
            TAG_NULL => b"NULL\n".to_vec(),
            TAG_INT => {
                format!("int({})\n", i64::from_ne_bytes(self.payload.to_ne_bytes())).into_bytes()
            }
            TAG_FLOAT => format!("float({})\n", f64::from_bits(self.payload)).into_bytes(),
            TAG_BOOL => format!(
                "bool({})\n",
                if self.as_bool() == Some(true) {
                    "true"
                } else {
                    "false"
                }
            )
            .into_bytes(),
            TAG_HEAP => match &self.heap_cell().data {
                HeapData::Bytes(bytes) => {
                    let display = String::from_utf8_lossy(bytes);
                    format!("string({}) \"{}\"\n", bytes.len(), display).into_bytes()
                }
                HeapData::Vector { values, .. } => {
                    format!("vector({})\n", values.len()).into_bytes()
                }
                HeapData::Map { entries, .. } => format!("map({})\n", entries.len()).into_bytes(),
                HeapData::Object { class, .. } => {
                    format!("object(class#{})\n", class.0).into_bytes()
                }
                HeapData::Stream { class, .. } => {
                    format!("stream(class#{})\n", class.0).into_bytes()
                }
                HeapData::Exception { class, .. } => {
                    format!("exception(class#{})\n", class.0).into_bytes()
                }
            },
            _ => b"<invalid>\n".to_vec(),
        }
    }

    fn make_heap_unique(&mut self) -> Result<(), RuntimeErrorKind> {
        if self.tag != TAG_HEAP {
            return Ok(());
        }
        let cell = self.heap_cell();
        if cell.references.get() == 1 {
            return Ok(());
        }
        let cloned = match &cell.data {
            HeapData::Vector {
                element_type,
                values,
            } => HeapData::Vector {
                element_type: element_type.clone(),
                values: values.clone(),
            },
            HeapData::Map {
                key_type,
                value_type,
                entries,
            } => HeapData::Map {
                key_type: key_type.clone(),
                value_type: value_type.clone(),
                entries: entries.clone(),
            },
            _ => return Ok(()),
        };
        let detached = if cell.owner.is_some() {
            Self::try_allocate(cloned)?
        } else {
            Self::allocate_unmanaged(cloned)
        };
        let previous = mem::replace(self, detached);
        drop(previous);
        Ok(())
    }

    fn heap_data(&self) -> Option<&HeapData> {
        (self.tag == TAG_HEAP).then(|| &self.heap_cell().data)
    }

    fn heap_data_mut(&mut self) -> Option<&mut HeapData> {
        if self.tag != TAG_HEAP {
            return None;
        }
        // SAFETY: `make_heap_unique` is called before mutation, and callers
        // hold `&mut self`, so this value cannot be concurrently accessed.
        let pointer = usize::try_from(self.payload)
            .expect("heap pointers fit the target pointer width")
            as *mut HeapCell;
        Some(unsafe { &mut (*pointer).data })
    }

    fn stream_state(&self) -> Result<&RefCell<StreamState>, RuntimeErrorKind> {
        match self.heap_data() {
            Some(HeapData::Stream { state, .. }) => Ok(state),
            _ => Err(RuntimeErrorKind::TypeError(
                "stream operation requires a stream object".to_owned(),
            )),
        }
    }

    fn heap_cell(&self) -> &HeapCell {
        debug_assert_eq!(self.tag, TAG_HEAP);
        // SAFETY: heap-tagged values are created only by `allocate`; their
        // pointer remains live while this Value owns one reference.
        let pointer = usize::try_from(self.payload)
            .expect("heap pointers fit the target pointer width")
            as *const HeapCell;
        unsafe { &*pointer }
    }

    fn heap_pointer(&self) -> *mut HeapCell {
        debug_assert_eq!(self.tag, TAG_HEAP);
        usize::try_from(self.payload).expect("heap pointers fit the target pointer width")
            as *mut HeapCell
    }

    fn owner_id(&self) -> Option<u64> {
        self.heap_data()
            .and_then(|_| self.heap_cell().owner.as_ref().map(|owner| owner.id))
    }

    fn ensure_same_owner(&self, other: &Self) -> Result<(), RuntimeErrorKind> {
        if self.tag != TAG_HEAP || other.tag != TAG_HEAP {
            return Ok(());
        }
        if self.owner_id() == other.owner_id() {
            Ok(())
        } else {
            Err(RuntimeErrorKind::CrossRequestValue)
        }
    }

    fn charge_growth(&self, bytes: usize) -> Result<(), RuntimeErrorKind> {
        if bytes == 0 || self.tag != TAG_HEAP {
            return Ok(());
        }
        let cell = self.heap_cell();
        if let Some(owner) = &cell.owner {
            owner.charge(bytes)?;
            cell.accounted_bytes
                .set(cell.accounted_bytes.get().saturating_add(bytes));
        }
        Ok(())
    }

    fn release_growth(&self, bytes: usize) {
        if bytes == 0 || self.tag != TAG_HEAP {
            return;
        }
        let cell = self.heap_cell();
        if let Some(owner) = &cell.owner {
            owner.release(bytes);
            cell.accounted_bytes
                .set(cell.accounted_bytes.get().saturating_sub(bytes));
        }
    }

    fn release_heap(&mut self) {
        if self.tag != TAG_HEAP {
            return;
        }
        let pointer = usize::try_from(self.payload)
            .expect("heap pointers fit the target pointer width")
            as *mut HeapCell;
        // SAFETY: the heap tag guarantees `pointer` came from Box::into_raw.
        // Request-thread confinement makes the Cell update race-free.
        let references = unsafe { (*pointer).references.get() };
        if references == 0 {
            self.tag = TAG_NULL;
            self.payload = 0;
            return;
        }
        if references == 1 {
            // SAFETY: the cell is live while this final Value owns it.
            let owner = unsafe { (*pointer).owner.as_ref().map(Rc::clone) };
            if let Some(owner) = &owner {
                if unsafe { (*pointer).buffered.get() } {
                    owner.unbuffer(pointer);
                }
                owner.release(unsafe { (*pointer).accounted_bytes.get() });
                owner
                    .live_cells
                    .set(owner.live_cells.get().saturating_sub(1));
            }
            // Mark non-heap before nested Value destructors run, preventing
            // accidental double release if a debugger observes this value.
            self.tag = TAG_NULL;
            self.payload = 0;
            // SAFETY: this is the last reference, so reconstructing and
            // dropping the Box happens exactly once.
            unsafe { drop(Box::from_raw(pointer)) };
        } else {
            // SAFETY: the allocation is still live and confined to this thread.
            unsafe { (*pointer).references.set(references - 1) };
            // SAFETY: the allocation remains live after the decrement.
            if let Some(owner) = unsafe { (*pointer).owner.as_ref() } {
                owner.buffer_candidate(pointer);
            }
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        if self.tag == TAG_HEAP {
            let cell = self.heap_cell();
            let references = cell.references.get();
            if references == u32::MAX {
                std::process::abort();
            }
            cell.references.set(references + 1);
        }
        Self {
            payload: self.payload,
            tag: self.tag,
            not_send_or_sync: PhantomData,
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        self.release_heap();
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tag {
            TAG_NULL => formatter.write_str("Null"),
            TAG_INT => formatter.debug_tuple("Int").field(&self.as_int()).finish(),
            TAG_FLOAT => formatter
                .debug_tuple("Float")
                .field(&self.as_float())
                .finish(),
            TAG_BOOL => formatter
                .debug_tuple("Bool")
                .field(&self.as_bool())
                .finish(),
            TAG_HEAP => self.heap_cell().data.fmt(formatter),
            tag => formatter.debug_tuple("Invalid").field(&tag).finish(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self.tag, other.tag) {
            (TAG_NULL, TAG_NULL) => true,
            (TAG_INT, TAG_INT) => self.as_int() == other.as_int(),
            (TAG_FLOAT, TAG_FLOAT) => self.as_float() == other.as_float(),
            (TAG_BOOL, TAG_BOOL) => self.as_bool() == other.as_bool(),
            (TAG_HEAP, TAG_HEAP) if self.payload == other.payload => true,
            (TAG_HEAP, TAG_HEAP) => match (&self.heap_cell().data, &other.heap_cell().data) {
                (HeapData::Bytes(left), HeapData::Bytes(right)) => left == right,
                (
                    HeapData::Vector {
                        element_type: left_type,
                        values: left,
                    },
                    HeapData::Vector {
                        element_type: right_type,
                        values: right,
                    },
                ) => left_type == right_type && left == right,
                (
                    HeapData::Map {
                        key_type: left_key,
                        value_type: left_value,
                        entries: left,
                    },
                    HeapData::Map {
                        key_type: right_key,
                        value_type: right_value,
                        entries: right,
                    },
                ) => left_key == right_key && left_value == right_value && left == right,
                _ => false,
            },
            _ => false,
        }
    }
}

fn format_output_float(value: f64) -> String {
    if value.is_nan() {
        return "NAN".to_owned();
    }
    if value == f64::INFINITY {
        return "INF".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-INF".to_owned();
    }

    let mut text = value.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    text
}

#[derive(Debug)]
enum HeapData {
    Bytes(Vec<u8>),
    Vector {
        element_type: Type,
        values: Vec<Value>,
    },
    Map {
        key_type: Type,
        value_type: Type,
        entries: Vec<(Value, Value)>,
    },
    Object {
        class: ClassId,
        properties: RefCell<Vec<Option<Value>>>,
    },
    Stream {
        class: ClassId,
        state: RefCell<StreamState>,
    },
    Exception {
        class: ClassId,
        properties: RefCell<Vec<Option<Value>>>,
        message: RefCell<Vec<u8>>,
        code: Cell<i64>,
        previous: RefCell<Option<Value>>,
        target: Option<Vec<u8>>,
        system_code: i64,
        suppressed: RefCell<Vec<Value>>,
    },
}

impl HeapData {
    fn dynamic_size(&self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.capacity(),
            Self::Vector { values, .. } => values.capacity() * mem::size_of::<Value>(),
            Self::Map { entries, .. } => entries.capacity() * mem::size_of::<(Value, Value)>(),
            Self::Object { properties, .. } => {
                properties.borrow().capacity() * mem::size_of::<Option<Value>>()
            }
            Self::Stream { state, .. } => match &state.borrow().storage {
                StreamStorage::Memory(bytes) => bytes.capacity(),
                StreamStorage::File { .. } | StreamStorage::Input(_) | StreamStorage::Closed => 0,
            },
            Self::Exception {
                properties,
                message,
                target,
                suppressed,
                ..
            } => {
                properties.borrow().capacity() * mem::size_of::<Option<Value>>()
                    + message.borrow().capacity()
                    + target.as_ref().map_or(0, Vec::capacity)
                    + suppressed.borrow().capacity() * mem::size_of::<Value>()
            }
        }
    }

    fn is_collectable(&self) -> bool {
        matches!(
            self,
            Self::Vector { .. } | Self::Map { .. } | Self::Object { .. } | Self::Exception { .. }
        )
    }

    fn for_each_collectable_child(&self, mut visit: impl FnMut(&Value)) {
        let mut visit_value = |value: &Value| {
            if value.tag == TAG_HEAP && value.heap_cell().data.is_collectable() {
                visit(value);
            }
        };
        match self {
            Self::Vector { values, .. } => {
                for value in values {
                    visit_value(value);
                }
            }
            Self::Map { entries, .. } => {
                for (key, value) in entries {
                    visit_value(key);
                    visit_value(value);
                }
            }
            Self::Object { properties, .. } => {
                for value in properties.borrow().iter().flatten() {
                    visit_value(value);
                }
            }
            Self::Exception {
                properties,
                previous,
                suppressed,
                ..
            } => {
                for value in properties.borrow().iter().flatten() {
                    visit_value(value);
                }
                if let Some(value) = previous.borrow().as_ref() {
                    visit_value(value);
                }
                for value in suppressed.borrow().iter() {
                    visit_value(value);
                }
            }
            Self::Bytes(_) | Self::Stream { .. } => {}
        }
    }

    fn clear_gc_edges(&mut self) {
        match self {
            Self::Vector { values, .. } => values.clear(),
            Self::Map { entries, .. } => entries.clear(),
            Self::Object { properties, .. } => properties.get_mut().clear(),
            Self::Exception {
                properties,
                previous,
                suppressed,
                ..
            } => {
                properties.get_mut().clear();
                drop(previous.get_mut().take());
                suppressed.get_mut().clear();
            }
            Self::Bytes(_) | Self::Stream { .. } => {}
        }
    }
}

#[derive(Debug)]
struct StreamState {
    storage: StreamStorage,
    cursor: usize,
    closed: bool,
    spill_threshold: Option<usize>,
    handle: Option<HandleLease>,
}

#[derive(Debug)]
enum StreamStorage {
    Memory(Vec<u8>),
    File { file: std::fs::File, length: usize },
    Input(RequestInput),
    Closed,
}

impl StreamState {
    fn len(&self) -> usize {
        match &self.storage {
            StreamStorage::Memory(bytes) => bytes.len(),
            StreamStorage::File { length, .. } => *length,
            StreamStorage::Input(_) | StreamStorage::Closed => self.cursor,
        }
    }

    fn read_range(&mut self, start: usize, end: usize) -> Result<Vec<u8>, RuntimeErrorKind> {
        match &mut self.storage {
            StreamStorage::Memory(bytes) => Ok(bytes[start..end].to_vec()),
            StreamStorage::File { file, .. } => {
                file.seek(SeekFrom::Start(start as u64))
                    .map_err(stream_io_error)?;
                let mut bytes = vec![0; end.saturating_sub(start)];
                file.read_exact(&mut bytes).map_err(stream_io_error)?;
                Ok(bytes)
            }
            StreamStorage::Input(_) => Err(RuntimeErrorKind::Io(
                "request input does not support ranged reads".to_owned(),
            )),
            StreamStorage::Closed => Err(RuntimeErrorKind::Io("stream is closed".to_owned())),
        }
    }

    fn spill_if_needed(&mut self, resulting_length: usize) -> Result<(), RuntimeErrorKind> {
        let should_spill = self
            .spill_threshold
            .is_some_and(|threshold| resulting_length > threshold)
            && matches!(self.storage, StreamStorage::Memory(_));
        if !should_spill {
            return Ok(());
        }
        let StreamStorage::Memory(bytes) =
            std::mem::replace(&mut self.storage, StreamStorage::Memory(Vec::new()))
        else {
            unreachable!("storage kind checked")
        };
        let mut file = tempfile::tempfile().map_err(stream_io_error)?;
        file.write_all(&bytes).map_err(stream_io_error)?;
        self.storage = StreamStorage::File {
            file,
            length: bytes.len(),
        };
        Ok(())
    }

    fn write_at_cursor(&mut self, bytes: &[u8]) -> Result<(), RuntimeErrorKind> {
        let end = self.cursor + bytes.len();
        match &mut self.storage {
            StreamStorage::Memory(storage) => {
                if end > storage.capacity() {
                    storage
                        .try_reserve_exact(end.saturating_sub(storage.len()))
                        .map_err(|_| RuntimeErrorKind::AllocationFailure)?;
                }
                if self.cursor > storage.len() {
                    storage.resize(self.cursor, 0);
                }
                if end > storage.len() {
                    storage.resize(end, 0);
                }
                storage[self.cursor..end].copy_from_slice(bytes);
            }
            StreamStorage::File { file, length } => {
                if self.cursor > *length {
                    file.set_len(self.cursor as u64).map_err(stream_io_error)?;
                }
                file.seek(SeekFrom::Start(self.cursor as u64))
                    .map_err(stream_io_error)?;
                file.write_all(bytes).map_err(stream_io_error)?;
                *length = (*length).max(end);
            }
            StreamStorage::Input(_) => {
                return Err(RuntimeErrorKind::Io(
                    "stream does not support writing".to_owned(),
                ));
            }
            StreamStorage::Closed => {
                return Err(RuntimeErrorKind::Io("stream is closed".to_owned()));
            }
        }
        Ok(())
    }
}

fn ensure_stream_open(state: &StreamState) -> Result<(), RuntimeErrorKind> {
    if state.closed {
        Err(RuntimeErrorKind::Io("stream is closed".to_owned()))
    } else {
        Ok(())
    }
}

#[allow(clippy::needless_pass_by_value)]
fn stream_io_error(error: std::io::Error) -> RuntimeErrorKind {
    RuntimeErrorKind::Io(format!("temporary stream I/O failed: {error}"))
}

struct HeapCell {
    references: Cell<u32>,
    gc_references: Cell<i64>,
    color: Cell<GcColor>,
    buffered: Cell<bool>,
    accounted_bytes: Cell<usize>,
    owner: Option<Rc<HeapState>>,
    data: HeapData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeErrorKind {
    Arithmetic(String),
    TypeError(String),
    Bounds(String),
    Io(String),
    HeapLimit { limit: usize },
    AllocationFailure,
    InputSizeLimit { limit: u64 },
    InputTimeLimit { limit: Duration },
    StackDepthLimit { limit: usize },
    OpenHandleLimit { limit: usize },
    OutputIo(String),
    CrossRequestValue,
    UninitializedLocal(u32),
    UninitializedProperty(u32),
    UncaughtException { class: String, message: String },
    Unreachable,
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arithmetic(message)
            | Self::TypeError(message)
            | Self::Bounds(message)
            | Self::Io(message)
            | Self::OutputIo(message) => formatter.write_str(message),
            Self::HeapLimit { limit } => {
                write!(
                    formatter,
                    "request exceeded the {limit} byte managed heap limit"
                )
            }
            Self::AllocationFailure => formatter.write_str("request allocation failed"),
            Self::InputSizeLimit { limit } => {
                write!(formatter, "request input exceeded the {limit} byte limit")
            }
            Self::InputTimeLimit { limit } => write!(
                formatter,
                "request input exceeded the {} second time limit",
                limit.as_secs_f64()
            ),
            Self::StackDepthLimit { limit } => {
                write!(formatter, "request exceeded the {limit} frame stack limit")
            }
            Self::OpenHandleLimit { limit } => {
                write!(formatter, "request exceeded the {limit} open handle limit")
            }
            Self::CrossRequestValue => {
                formatter.write_str("heap values cannot cross request ownership boundaries")
            }
            Self::UninitializedLocal(local) => {
                write!(formatter, "local {local} was read before initialization")
            }
            Self::UninitializedProperty(property) => {
                write!(
                    formatter,
                    "property {property} was read before initialization"
                )
            }
            Self::UncaughtException { class, message } => {
                write!(formatter, "uncaught {class}")?;
                if !message.is_empty() {
                    write!(formatter, ": {message}")?;
                }
                Ok(())
            }
            Self::Unreachable => formatter.write_str("execution reached unreachable bytecode"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StackFrame {
    pub function: String,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub span: Span,
    pub trace: Vec<StackFrame>,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
            trace: Vec::new(),
        }
    }

    pub fn push_frame(&mut self, function: impl Into<String>, span: Span) {
        self.trace.push(StackFrame {
            function: function.into(),
            span,
        });
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "runtime error: {}", self.kind)?;
        for frame in &self.trace {
            write!(
                formatter,
                "\n  at {} (source bytes {}..{})",
                frame.function, frame.span.start, frame.span.end
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::mem;
    use std::time::Duration;

    use thp_hir::Type;

    use super::{HeapData, RequestHeap, RequestInput, RuntimeErrorKind, StreamStorage, Value};

    #[test]
    fn values_use_two_words() {
        assert_eq!(mem::size_of::<Value>(), 16);
    }

    #[test]
    fn heap_values_share_and_release() {
        let value = Value::bytes(b"hello".to_vec());
        let alias = value.clone();
        assert_eq!(value, alias);
        drop(value);
        assert_eq!(alias.as_bytes(), Some(b"hello".as_slice()));
    }

    #[test]
    fn vector_mutation_detaches_shared_storage() {
        let mut left = Value::vector(Type::Int, vec![Value::integer(1)]);
        let right = left.clone();
        left.vector_push(Value::integer(2)).unwrap();
        assert_eq!(left.count(), Some(2));
        assert_eq!(right.count(), Some(1));
    }

    #[test]
    fn collection_iteration_and_assignment_preserve_map_order_and_copy_on_write() {
        let mut left = Value::map(
            Type::String,
            Type::Int,
            vec![(Value::bytes(b"first".to_vec()), Value::integer(1))],
        );
        let right = left.clone();
        left.set_index(&Value::bytes(b"first".to_vec()), Value::integer(2))
            .unwrap();
        left.set_index(&Value::bytes(b"second".to_vec()), Value::integer(3))
            .unwrap();

        assert_eq!(left.collection_len().unwrap(), 2);
        assert_eq!(
            left.collection_key_at(1).unwrap().as_bytes(),
            Some(b"second".as_slice())
        );
        assert_eq!(left.collection_value_at(1).unwrap().as_int(), Some(3));
        assert_eq!(
            right
                .index(&Value::bytes(b"first".to_vec()))
                .unwrap()
                .as_int(),
            Some(1)
        );
        assert_eq!(right.collection_len().unwrap(), 1);
    }

    #[test]
    fn strings_preserve_arbitrary_bytes() {
        let bytes = Value::bytes(vec![0, 0xff, b'T']);
        assert_eq!(bytes.as_bytes(), Some([0, 0xff, b'T'].as_slice()));
    }

    #[test]
    fn output_scalars_have_canonical_text() {
        assert_eq!(
            Value::integer(i64::MIN).output_bytes().unwrap(),
            b"-9223372036854775808"
        );
        assert_eq!(Value::float(1.0).output_bytes().unwrap(), b"1.0");
        assert_eq!(Value::float(-0.0).output_bytes().unwrap(), b"-0.0");
        assert_eq!(Value::float(f64::NAN).output_bytes().unwrap(), b"NAN");
        assert_eq!(Value::float(f64::INFINITY).output_bytes().unwrap(), b"INF");
        assert_eq!(
            Value::float(f64::NEG_INFINITY).output_bytes().unwrap(),
            b"-INF"
        );
        assert_eq!(Value::bool(true).output_bytes().unwrap(), b"true");
        assert_eq!(Value::bool(false).output_bytes().unwrap(), b"false");
        assert_eq!(Value::bytes([0, 0xff]).output_bytes().unwrap(), [0, 0xff]);
    }

    #[test]
    fn null_and_compound_values_are_not_output_scalars() {
        assert!(Value::NULL.output_bytes().is_none());
        assert!(
            Value::vector(Type::Int, Vec::new())
                .output_bytes()
                .is_none()
        );
    }

    #[test]
    fn object_aliases_share_property_mutation() {
        let object = Value::object(thp_hir::ClassId(3), 1);
        let alias = object.clone();
        object
            .set_property(thp_hir::PropertyId(0), Value::integer(7))
            .unwrap();
        assert_eq!(
            alias.property(thp_hir::PropertyId(0)).unwrap().as_int(),
            Some(7)
        );
        assert_eq!(alias.class_id(), Some(thp_hir::ClassId(3)));
    }

    #[test]
    fn throwable_state_and_previous_chains_are_shared() {
        let root = Value::throwable_object(thp_hir::ClassId(10), 0);
        root.initialize_exception(b"root".to_vec(), 1, None)
            .unwrap();
        let existing = Value::throwable_object(thp_hir::ClassId(10), 0);
        existing
            .initialize_exception(b"existing".to_vec(), 2, Some(root.clone()))
            .unwrap();
        let replacement = Value::throwable_object(thp_hir::ClassId(10), 0);
        replacement
            .initialize_exception(b"replacement".to_vec(), 3, Some(existing.clone()))
            .unwrap();
        let pending = Value::throwable_object(thp_hir::ClassId(10), 0);
        pending
            .initialize_exception(b"pending".to_vec(), 4, None)
            .unwrap();

        replacement.append_previous(pending.clone()).unwrap();
        assert_eq!(replacement.exception_message().unwrap(), b"replacement");
        assert_eq!(replacement.exception_code().unwrap(), 3);
        assert_eq!(replacement.exception_previous().unwrap(), existing);
        assert_eq!(root.exception_previous().unwrap(), pending);

        replacement.append_previous(replacement.clone()).unwrap();
        assert_eq!(root.exception_previous().unwrap(), pending);

        let cycle_root = Value::throwable_object(thp_hir::ClassId(10), 0);
        let cycle_pending = Value::throwable_object(thp_hir::ClassId(10), 0);
        cycle_pending
            .initialize_exception(b"cycle".to_vec(), 0, Some(cycle_root.clone()))
            .unwrap();
        cycle_root.append_previous(cycle_pending).unwrap();
        assert!(cycle_root.exception_previous().unwrap().is_null());
    }

    #[test]
    fn temporary_stream_spills_to_an_unnamed_file() {
        let stream = Value::temp_stream(thp_hir::ClassId(4), 0);
        stream.stream_write_all(b"spill").unwrap();
        let Some(HeapData::Stream { state, .. }) = stream.heap_data() else {
            panic!("expected stream heap data");
        };
        assert!(matches!(state.borrow().storage, StreamStorage::File { .. }));
        stream.stream_seek(0).unwrap();
        assert_eq!(stream.stream_read_all(None).unwrap(), b"spill");
    }

    #[test]
    fn request_heap_collects_self_and_mutual_cycles() {
        let heap = RequestHeap::new(Some(1024 * 1024), None).unwrap();
        let _active = heap.activate();

        let self_cycle = Value::try_object(thp_hir::ClassId(1), 1).unwrap();
        self_cycle
            .set_property(thp_hir::PropertyId(0), self_cycle.clone())
            .unwrap();
        drop(self_cycle);

        let left = Value::try_object(thp_hir::ClassId(1), 1).unwrap();
        let right = Value::try_object(thp_hir::ClassId(1), 1).unwrap();
        left.set_property(thp_hir::PropertyId(0), right.clone())
            .unwrap();
        right
            .set_property(thp_hir::PropertyId(0), left.clone())
            .unwrap();
        drop(left);
        drop(right);

        assert_eq!(heap.stats().live_cells, 3);
        assert_eq!(heap.collect_cycles(), 3);
        assert_eq!(heap.stats().live_cells, 0);
        assert_eq!(heap.stats().collected_cells, 3);
    }

    #[test]
    fn reachable_cycle_survives_until_its_external_root_is_dropped() {
        let heap = RequestHeap::new(Some(1024 * 1024), None).unwrap();
        let _active = heap.activate();
        let value = Value::try_object(thp_hir::ClassId(1), 1).unwrap();
        value
            .set_property(thp_hir::PropertyId(0), value.clone())
            .unwrap();

        assert_eq!(heap.collect_cycles(), 0);
        assert_eq!(heap.stats().live_cells, 1);
        drop(value);
        assert_eq!(heap.collect_cycles(), 1);
        assert_eq!(heap.stats().live_cells, 0);
    }

    #[test]
    fn request_heap_reports_limits_and_injected_allocation_failure() {
        let heap = RequestHeap::new(Some(4096), None).unwrap();
        let _active = heap.activate();
        let error = Value::try_bytes(vec![0; 8192]).unwrap_err();
        assert!(matches!(error, RuntimeErrorKind::HeapLimit { limit: 4096 }));

        heap.fail_allocations_after(0);
        assert_eq!(
            Value::try_bytes(Vec::new()).unwrap_err(),
            RuntimeErrorKind::AllocationFailure
        );
    }

    #[test]
    fn stream_handle_limit_counts_cells_not_aliases_and_close_releases() {
        let heap = RequestHeap::new(Some(1024 * 1024), Some(1)).unwrap();
        let _active = heap.activate();
        let stream = Value::try_stream(thp_hir::ClassId(4), Vec::new()).unwrap();
        let alias = stream.clone();
        assert_eq!(heap.stats().open_handles, 1);
        assert!(matches!(
            Value::try_stream(thp_hir::ClassId(4), Vec::new()),
            Err(RuntimeErrorKind::OpenHandleLimit { limit: 1 })
        ));
        alias.stream_close().unwrap();
        assert_eq!(heap.stats().open_handles, 0);
        drop(stream);
        drop(alias);
        Value::try_stream(thp_hir::ClassId(4), Vec::new()).unwrap();
    }

    #[test]
    fn request_input_enforces_declared_and_streamed_size_and_shared_cursor() {
        assert!(matches!(
            RequestInput::new(Box::new(Cursor::new(vec![1, 2])), Some(2), Some(1), None),
            Err(RuntimeErrorKind::InputSizeLimit { limit: 1 })
        ));

        let input = RequestInput::new(
            Box::new(Cursor::new(b"abcd".to_vec())),
            None,
            Some(3),
            Some(Duration::from_secs(1)),
        )
        .unwrap();
        let heap = RequestHeap::new(Some(1024 * 1024), Some(1)).unwrap();
        let _active = heap.activate();
        let stream = Value::try_request_input_stream(thp_hir::ClassId(4), input.clone()).unwrap();
        let alias = stream.clone();
        assert_eq!(stream.stream_read(2).unwrap(), b"ab");
        assert_eq!(alias.stream_tell().unwrap(), 2);
        assert!(matches!(
            alias.stream_read(2),
            Err(RuntimeErrorKind::InputSizeLimit { limit: 3 })
        ));
    }

    #[test]
    fn request_input_read_all_limit_preserves_logical_cursor() {
        let input = RequestInput::from_bytes(b"abcdef".to_vec(), Some(16), None).unwrap();
        let heap = RequestHeap::new(Some(1024 * 1024), Some(1)).unwrap();
        let _active = heap.activate();
        let stream = Value::try_request_input_stream(thp_hir::ClassId(4), input).unwrap();
        assert!(matches!(
            stream.stream_read_all(Some(3)),
            Err(RuntimeErrorKind::Io(message)) if message == "stream read limit exceeded"
        ));
        assert_eq!(stream.stream_tell().unwrap(), 0);
        assert_eq!(stream.stream_read(6).unwrap(), b"abcdef");
    }

    #[test]
    fn request_input_time_limit_is_non_io_policy_failure() {
        let input = RequestInput::from_bytes(b"x".to_vec(), None, Some(Duration::ZERO)).unwrap();
        let heap = RequestHeap::new(Some(1024 * 1024), Some(1)).unwrap();
        let _active = heap.activate();
        let stream = Value::try_request_input_stream(thp_hir::ClassId(4), input).unwrap();
        assert!(matches!(
            stream.stream_read(1),
            Err(RuntimeErrorKind::InputTimeLimit { limit }) if limit.is_zero()
        ));
    }

    #[test]
    fn collector_stress_reclaims_many_cycles_across_repeated_requests() {
        for _request in 0..8 {
            let heap = RequestHeap::new(Some(4 * 1024 * 1024), Some(32)).unwrap();
            let _active = heap.activate();
            for _cycle in 0..1000 {
                let value = Value::try_object(thp_hir::ClassId(1), 1).unwrap();
                value
                    .set_property(thp_hir::PropertyId(0), value.clone())
                    .unwrap();
                drop(value);
            }
            assert_eq!(heap.collect_cycles(), 1000);
            assert_eq!(heap.stats().live_cells, 0);
            assert_eq!(heap.stats().open_handles, 0);
        }
    }
}
