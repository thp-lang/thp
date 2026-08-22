//! Stable C-facing THP embedding and extension ABI.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_void};
use std::io::{Read, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::time::Duration;
use std::{io, slice};

use thp_compiler::ProjectRequest;
use thp_diagnostics::SourceFile;
use thp_embed::{Engine, PreparedProject, Status};
use thp_modules::{ModuleError, ModuleId, ModulePath, ModuleSourceProvider};
use thp_runtime::RequestInput;
use thp_vm::{ExecutionContext, Limits};

pub const THP_ABI_VERSION: u32 = 1;

#[repr(C)]
pub struct ThpEngine {
    engine: Engine,
    limits: Limits,
}

#[repr(C)]
pub struct ThpPreparedProject {
    prepared: PreparedProject,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ThpBuffer {
    pub pointer: *mut u8,
    pub length: usize,
    pub capacity: usize,
}

impl ThpBuffer {
    const EMPTY: Self = Self {
        pointer: std::ptr::null_mut(),
        length: 0,
        capacity: 0,
    };

    fn from_vec(mut bytes: Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::EMPTY;
        }
        let buffer = Self {
            pointer: bytes.as_mut_ptr(),
            length: bytes.len(),
            capacity: bytes.capacity(),
        };
        std::mem::forget(bytes);
        buffer
    }
}

#[repr(C)]
pub struct ThpRunResult {
    pub status: u32,
    pub output: ThpBuffer,
    pub error: ThpBuffer,
}

#[repr(C)]
pub struct ThpEngineOptions {
    pub abi_version: u32,
    pub struct_size: usize,
    pub max_instructions: u64,
    pub max_execution_ms: u64,
    pub max_heap_bytes: u64,
    pub max_input_bytes: u64,
    pub max_input_ms: u64,
    pub max_stack_depth: u64,
    pub max_open_handles: u64,
}

#[repr(C)]
pub struct ThpEngineCreateResult {
    pub engine: *mut ThpEngine,
    pub error: ThpBuffer,
}

pub type ThpInputReadFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    buffer: *mut u8,
    capacity: usize,
    length: *mut usize,
) -> i32;
pub type ThpOutputWriteFn =
    unsafe extern "C" fn(user_data: *mut c_void, buffer: *const u8, length: usize) -> i32;

#[repr(C)]
pub struct ThpIo {
    pub abi_version: u32,
    pub struct_size: usize,
    pub input_read: Option<ThpInputReadFn>,
    pub output_write: Option<ThpOutputWriteFn>,
    pub declared_input_length: u64,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct ThpStreamingResult {
    pub status: u32,
    pub error: ThpBuffer,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

impl ThpStreamingResult {
    fn host_error(message: impl Into<Vec<u8>>) -> Self {
        Self {
            status: Status::HostError as u32,
            error: ThpBuffer::from_vec(message.into()),
            input_bytes: 0,
            output_bytes: 0,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ThpBorrowedBuffer {
    pub pointer: *const u8,
    pub length: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ThpModuleDescriptor {
    pub module_id: ThpBorrowedBuffer,
    pub path: ThpBorrowedBuffer,
    pub expected_namespace: ThpBorrowedBuffer,
    pub is_entry: u8,
}

pub type ThpModuleEnumerateFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    index: usize,
    descriptor: *mut ThpModuleDescriptor,
) -> i32;
pub type ThpModuleLoadFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    module_id: ThpBorrowedBuffer,
    source: *mut ThpBorrowedBuffer,
) -> i32;

#[repr(C)]
pub struct ThpModuleProvider {
    pub abi_version: u32,
    pub struct_size: usize,
    pub enumerate: Option<ThpModuleEnumerateFn>,
    pub load: Option<ThpModuleLoadFn>,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct ThpProjectOptions {
    pub abi_version: u32,
    pub struct_size: usize,
    pub project_root: ThpBorrowedBuffer,
    pub entry: ThpBorrowedBuffer,
    pub target: ThpBorrowedBuffer,
}

#[repr(C)]
pub struct ThpPrepareResult {
    pub status: u32,
    pub project: *mut ThpPreparedProject,
    pub error: ThpBuffer,
}

impl ThpPrepareResult {
    fn host_error(message: impl Into<Vec<u8>>) -> Self {
        Self {
            status: Status::HostError as u32,
            project: std::ptr::null_mut(),
            error: ThpBuffer::from_vec(message.into()),
        }
    }
}

impl ThpRunResult {
    fn host_error(message: impl Into<Vec<u8>>) -> Self {
        Self {
            status: Status::HostError as u32,
            output: ThpBuffer::EMPTY,
            error: ThpBuffer::from_vec(message.into()),
        }
    }
}

/// Creates an isolated THP engine.
#[unsafe(no_mangle)]
pub extern "C" fn thp_engine_new(max_instructions: u64) -> *mut ThpEngine {
    let limits = Limits {
        max_instructions: (max_instructions != 0).then_some(max_instructions),
        max_execution: None,
        ..Limits::default()
    };
    Box::into_raw(Box::new(ThpEngine {
        engine: Engine::new(limits),
        limits,
    }))
}

/// Creates an engine with all request limits configured explicitly.
///
/// # Safety
///
/// `options` must point to a live ABI-v1 options structure for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_new_with_options(
    options: *const ThpEngineOptions,
) -> ThpEngineCreateResult {
    catch_unwind(AssertUnwindSafe(|| {
        if options.is_null() {
            return ThpEngineCreateResult {
                engine: std::ptr::null_mut(),
                error: ThpBuffer::from_vec(b"THP engine options pointer is null".to_vec()),
            };
        }
        // SAFETY: pointer validity is the caller invariant.
        let options = unsafe { &*options };
        if options.abi_version != THP_ABI_VERSION
            || options.struct_size < std::mem::size_of::<ThpEngineOptions>()
        {
            return ThpEngineCreateResult {
                engine: std::ptr::null_mut(),
                error: ThpBuffer::from_vec(
                    b"unsupported ABI version or truncated engine options".to_vec(),
                ),
            };
        }
        let usize_limit = |value: u64| (value != 0).then(|| usize::try_from(value).ok()).flatten();
        if [
            options.max_heap_bytes,
            options.max_stack_depth,
            options.max_open_handles,
        ]
        .into_iter()
        .any(|value| value != 0 && usize::try_from(value).is_err())
        {
            return ThpEngineCreateResult {
                engine: std::ptr::null_mut(),
                error: ThpBuffer::from_vec(b"engine limit does not fit this platform".to_vec()),
            };
        }
        let limits = Limits {
            max_instructions: (options.max_instructions != 0).then_some(options.max_instructions),
            max_execution: (options.max_execution_ms != 0)
                .then(|| Duration::from_millis(options.max_execution_ms)),
            max_heap_bytes: usize_limit(options.max_heap_bytes),
            max_input_bytes: (options.max_input_bytes != 0).then_some(options.max_input_bytes),
            max_input_time: (options.max_input_ms != 0)
                .then(|| Duration::from_millis(options.max_input_ms)),
            max_stack_depth: usize_limit(options.max_stack_depth),
            max_open_handles: usize_limit(options.max_open_handles),
        };
        ThpEngineCreateResult {
            engine: Box::into_raw(Box::new(ThpEngine {
                engine: Engine::new(limits),
                limits,
            })),
            error: ThpBuffer::EMPTY,
        }
    }))
    .unwrap_or_else(|_| ThpEngineCreateResult {
        engine: std::ptr::null_mut(),
        error: ThpBuffer::from_vec(b"panic crossed the THP host boundary".to_vec()),
    })
}

/// Destroys an engine returned by [`thp_engine_new`].
///
/// # Safety
///
/// `engine` must be null or a live pointer returned by `thp_engine_new`, and
/// it must not be freed more than once or used after this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_free(engine: *mut ThpEngine) {
    if !engine.is_null() {
        // SAFETY: guaranteed by the caller contract above.
        unsafe { drop(Box::from_raw(engine)) };
    }
}

/// Compiles and executes one UTF-8 source buffer.
///
/// The returned buffers belong to THP and must each be released with
/// [`thp_buffer_free`].
///
/// # Safety
///
/// `engine` must point to a live `ThpEngine`. Non-empty path/source buffers
/// must be readable for their stated lengths for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_run(
    engine: *mut ThpEngine,
    path: *const u8,
    path_length: usize,
    source: *const u8,
    source_length: usize,
) -> ThpRunResult {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() {
            return ThpRunResult::host_error("THP engine pointer is null");
        }
        // SAFETY: checked for null and covered by this function's contract.
        let engine = unsafe { &*engine };
        // SAFETY: buffer validity is part of `thp_engine_run`'s caller
        // contract, and the borrowed slice does not escape this call.
        let path = match unsafe { borrowed_bytes(path, path_length) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| "path is not UTF-8"))
        {
            Ok(path) => path,
            Err(message) => return ThpRunResult::host_error(message),
        };
        // SAFETY: the same source-buffer invariant applies here.
        let source = match unsafe { borrowed_bytes(source, source_length) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| "source is not UTF-8"))
        {
            Ok(source) => source,
            Err(message) => return ThpRunResult::host_error(message),
        };
        let response = engine.engine.execute(path, source);
        ThpRunResult {
            status: response.status as u32,
            output: ThpBuffer::from_vec(response.output),
            error: ThpBuffer::from_vec(response.error),
        }
    }));
    result.unwrap_or_else(|_| ThpRunResult::host_error("panic crossed the THP host boundary"))
}

struct CallbackReader {
    callback: Option<ThpInputReadFn>,
    user_data: *mut c_void,
}

impl Read for CallbackReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let Some(callback) = self.callback else {
            return Ok(0);
        };
        let mut length = 0usize;
        // SAFETY: the callback and user data are supplied in a validated
        // `ThpIo`; the mutable buffer is live for this synchronous call.
        let status = unsafe {
            callback(
                self.user_data,
                buffer.as_mut_ptr(),
                buffer.len(),
                &raw mut length,
            )
        };
        if status != 0 {
            return Err(io::Error::other(format!(
                "input callback failed with status {status}"
            )));
        }
        if length > buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "input callback returned an oversized length",
            ));
        }
        Ok(length)
    }
}

struct CallbackWriter {
    callback: ThpOutputWriteFn,
    user_data: *mut c_void,
}

impl Write for CallbackWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        // SAFETY: the callback and user data are supplied in a validated
        // `ThpIo`; the immutable buffer is live for this synchronous call.
        let status = unsafe { (self.callback)(self.user_data, buffer.as_ptr(), buffer.len()) };
        if status != 0 {
            return Err(io::Error::other(format!(
                "output callback failed with status {status}"
            )));
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

unsafe fn callback_io(
    engine: &ThpEngine,
    io: *const ThpIo,
) -> Result<(ExecutionContext, CallbackWriter), &'static str> {
    if io.is_null() {
        return Err("THP I/O pointer is null");
    }
    // SAFETY: pointer validity is the caller invariant.
    let io = unsafe { &*io };
    if io.abi_version != THP_ABI_VERSION || io.struct_size < std::mem::size_of::<ThpIo>() {
        return Err("unsupported ABI version or truncated I/O structure");
    }
    let output = io.output_write.ok_or("output callback is missing")?;
    let declared_length =
        (io.declared_input_length != u64::MAX).then_some(io.declared_input_length);
    let input = RequestInput::new(
        Box::new(CallbackReader {
            callback: io.input_read,
            user_data: io.user_data,
        }),
        declared_length,
        engine.limits.max_input_bytes,
        engine.limits.max_input_time,
    )
    .map_err(|_| "declared input exceeds the configured request limit")?;
    Ok((
        ExecutionContext {
            limits: engine.limits,
            filesystem_base: PathBuf::new(),
            request_input: input,
        },
        CallbackWriter {
            callback: output,
            user_data: io.user_data,
        },
    ))
}

/// Compiles and executes one source buffer with callback-driven input/output.
///
/// # Safety
///
/// All pointers and callback-owned state must remain live for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_run_io(
    engine: *mut ThpEngine,
    path: *const u8,
    path_length: usize,
    source: *const u8,
    source_length: usize,
    io: *const ThpIo,
) -> ThpStreamingResult {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() {
            return ThpStreamingResult::host_error("THP engine pointer is null");
        }
        // SAFETY: checked non-null and covered by the function contract.
        let engine = unsafe { &*engine };
        // SAFETY: borrowed buffers remain live for this call.
        let path = match unsafe { borrowed_bytes(path, path_length) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| "path is not UTF-8"))
        {
            Ok(path) => path,
            Err(message) => return ThpStreamingResult::host_error(message),
        };
        // SAFETY: same source-buffer invariant.
        let source = match unsafe { borrowed_bytes(source, source_length) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| "source is not UTF-8"))
        {
            Ok(source) => source,
            Err(message) => return ThpStreamingResult::host_error(message),
        };
        // SAFETY: callback structure validity is the caller invariant.
        let (context, mut writer) = match unsafe { callback_io(engine, io) } {
            Ok(io) => io,
            Err(message) => return ThpStreamingResult::host_error(message),
        };
        let response = engine
            .engine
            .execute_to(path, source, &context, &mut writer);
        ThpStreamingResult {
            status: response.status as u32,
            error: ThpBuffer::from_vec(response.error),
            input_bytes: response.request.input_bytes,
            output_bytes: response.request.output_bytes,
        }
    }))
    .unwrap_or_else(|_| ThpStreamingResult::host_error("panic crossed the THP host boundary"))
}

/// Compiles a complete project supplied by synchronous host callbacks.
///
/// Provider buffers are borrowed only for the duration of their callback and
/// copied before control returns to the host.
///
/// # Safety
///
/// All pointers must be null or point to live ABI-v1 project structures for
/// the duration of this call. Callback-returned non-empty buffers must be
/// readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_prepare_project(
    engine: *mut ThpEngine,
    provider: *const ThpModuleProvider,
    options: *const ThpProjectOptions,
) -> ThpPrepareResult {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || provider.is_null() || options.is_null() {
            return ThpPrepareResult::host_error("null project preparation pointer");
        }
        // SAFETY: non-null pointers and structure lifetimes are caller invariants.
        let engine = unsafe { &*engine };
        // SAFETY: covered by the same caller contract.
        let provider = unsafe { &*provider };
        // SAFETY: covered by the same caller contract.
        let options = unsafe { &*options };
        if provider.abi_version != THP_ABI_VERSION
            || provider.struct_size < std::mem::size_of::<ThpModuleProvider>()
            || options.abi_version != THP_ABI_VERSION
            || options.struct_size < std::mem::size_of::<ThpProjectOptions>()
        {
            return ThpPrepareResult::host_error(
                "unsupported ABI version or truncated project structure",
            );
        }
        let root = match unsafe { copied_utf8(options.project_root) } {
            Ok(value) => PathBuf::from(value),
            Err(message) => return ThpPrepareResult::host_error(message),
        };
        let entry = match unsafe { copied_utf8(options.entry) } {
            Ok(value) => PathBuf::from(value),
            Err(message) => return ThpPrepareResult::host_error(message),
        };
        let target = match unsafe { copied_utf8(options.target) } {
            Ok(value) if value.is_empty() => None,
            Ok(value) => Some(value),
            Err(message) => return ThpPrepareResult::host_error(message),
        };
        let callback_provider = match unsafe { CallbackProvider::new(provider) } {
            Ok(provider) => provider,
            Err(message) => return ThpPrepareResult::host_error(message),
        };
        let request = ProjectRequest {
            project_root: root,
            entry,
            target,
        };
        match engine.engine.prepare_project(&request, &callback_provider) {
            Ok(prepared) => ThpPrepareResult {
                status: Status::Success as u32,
                project: Box::into_raw(Box::new(ThpPreparedProject { prepared })),
                error: ThpBuffer::EMPTY,
            },
            Err(response) => ThpPrepareResult {
                status: response.status as u32,
                project: std::ptr::null_mut(),
                error: ThpBuffer::from_vec(response.error),
            },
        }
    }))
    .unwrap_or_else(|_| ThpPrepareResult::host_error("panic crossed the THP host boundary"))
}

/// Executes immutable verified code retained by a prepared project.
///
/// # Safety
///
/// Both pointers must remain live for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_run_prepared(
    engine: *mut ThpEngine,
    project: *const ThpPreparedProject,
) -> ThpRunResult {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || project.is_null() {
            return ThpRunResult::host_error("null prepared execution pointer");
        }
        // SAFETY: pointer validity is the caller invariant.
        let engine = unsafe { &*engine };
        // SAFETY: pointer validity is the caller invariant.
        let project = unsafe { &*project };
        let response = engine.engine.execute_prepared(&project.prepared);
        ThpRunResult {
            status: response.status as u32,
            output: ThpBuffer::from_vec(response.output),
            error: ThpBuffer::from_vec(response.error),
        }
    }))
    .unwrap_or_else(|_| ThpRunResult::host_error("panic crossed the THP host boundary"))
}

/// Executes prepared code with callback-driven input/output.
///
/// # Safety
///
/// All pointers and callback-owned state must remain live for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_engine_run_prepared_io(
    engine: *mut ThpEngine,
    project: *const ThpPreparedProject,
    io: *const ThpIo,
) -> ThpStreamingResult {
    catch_unwind(AssertUnwindSafe(|| {
        if engine.is_null() || project.is_null() {
            return ThpStreamingResult::host_error("null prepared execution pointer");
        }
        // SAFETY: pointer validity is the caller invariant.
        let engine = unsafe { &*engine };
        // SAFETY: pointer validity is the caller invariant.
        let project = unsafe { &*project };
        // SAFETY: callback structure validity is the caller invariant.
        let (context, mut writer) = match unsafe { callback_io(engine, io) } {
            Ok(io) => io,
            Err(message) => return ThpStreamingResult::host_error(message),
        };
        let response = engine
            .engine
            .execute_prepared_to(&project.prepared, &context, &mut writer);
        ThpStreamingResult {
            status: response.status as u32,
            error: ThpBuffer::from_vec(response.error),
            input_bytes: response.request.input_bytes,
            output_bytes: response.request.output_bytes,
        }
    }))
    .unwrap_or_else(|_| ThpStreamingResult::host_error("panic crossed the THP host boundary"))
}

/// Releases a prepared project.
///
/// # Safety
///
/// The pointer must be null or returned by `thp_engine_prepare_project`, and
/// must be freed at most once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_prepared_project_free(project: *mut ThpPreparedProject) {
    if !project.is_null() {
        // SAFETY: guaranteed by the caller contract.
        unsafe { drop(Box::from_raw(project)) };
    }
}

unsafe fn borrowed_bytes<'buffer>(
    pointer: *const u8,
    length: usize,
) -> Result<&'buffer [u8], &'static str> {
    if length == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err("non-empty buffer has a null pointer");
    }
    // SAFETY: non-null readability for `length` bytes is the caller invariant.
    Ok(unsafe { slice::from_raw_parts(pointer, length) })
}

unsafe fn copied_utf8(buffer: ThpBorrowedBuffer) -> Result<String, &'static str> {
    // SAFETY: the callback/option buffer contract is forwarded here.
    let bytes = unsafe { borrowed_bytes(buffer.pointer, buffer.length) }?;
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| "host buffer is not UTF-8")
}

struct CallbackProvider {
    modules: Vec<ModulePath>,
    load: ThpModuleLoadFn,
    user_data: *mut c_void,
}

impl CallbackProvider {
    unsafe fn new(provider: &ThpModuleProvider) -> Result<Self, String> {
        let enumerate = provider
            .enumerate
            .ok_or_else(|| "module enumerate callback is missing".to_owned())?;
        let load = provider
            .load
            .ok_or_else(|| "module load callback is missing".to_owned())?;
        let mut modules = Vec::new();
        for index in 0.. {
            let mut descriptor = ThpModuleDescriptor {
                module_id: ThpBorrowedBuffer {
                    pointer: std::ptr::null(),
                    length: 0,
                },
                path: ThpBorrowedBuffer {
                    pointer: std::ptr::null(),
                    length: 0,
                },
                expected_namespace: ThpBorrowedBuffer {
                    pointer: std::ptr::null(),
                    length: 0,
                },
                is_entry: 0,
            };
            // SAFETY: callback validity and output initialization are the host contract.
            let status = unsafe { enumerate(provider.user_data, index, &raw mut descriptor) };
            if status == 1 {
                break;
            }
            if status != 0 {
                return Err(format!(
                    "module enumerate callback failed at index {index} with status {status}"
                ));
            }
            // SAFETY: descriptor buffers remain borrowed until this callback return
            // boundary; each is copied immediately.
            let module_id = unsafe { copied_utf8(descriptor.module_id) }
                .map_err(str::to_owned)
                .and_then(|value| ModuleId::new(value).map_err(|error| error.to_string()))?;
            // SAFETY: same descriptor buffer contract.
            let path =
                PathBuf::from(unsafe { copied_utf8(descriptor.path) }.map_err(str::to_owned)?);
            // SAFETY: same descriptor buffer contract.
            let expected_namespace =
                unsafe { copied_utf8(descriptor.expected_namespace) }.map_err(str::to_owned)?;
            modules.push(ModulePath {
                id: module_id,
                canonical_path: path.clone(),
                path,
                expected_namespace,
                is_entry: descriptor.is_entry != 0,
            });
        }
        if modules.iter().filter(|module| module.is_entry).count() != 1 {
            return Err("module provider must enumerate exactly one entry".to_owned());
        }
        modules.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            modules,
            load,
            user_data: provider.user_data,
        })
    }
}

impl ModuleSourceProvider for CallbackProvider {
    fn enumerate(&self) -> Result<Vec<ModulePath>, ModuleError> {
        Ok(self.modules.clone())
    }

    fn load(&self, module: &ModulePath) -> Result<SourceFile, ModuleError> {
        let id = module.id.as_str().as_bytes();
        let mut source = ThpBorrowedBuffer {
            pointer: std::ptr::null(),
            length: 0,
        };
        // SAFETY: callback was validated during provider construction; the ID
        // buffer remains live for the call and the returned source is copied.
        let status = unsafe {
            (self.load)(
                self.user_data,
                ThpBorrowedBuffer {
                    pointer: id.as_ptr(),
                    length: id.len(),
                },
                &raw mut source,
            )
        };
        if status != 0 {
            return Err(ModuleError::Io {
                path: module.path.clone(),
                source: io::Error::other(format!(
                    "module load callback failed with status {status}"
                )),
            });
        }
        // SAFETY: the callback promises this borrow until it returns; copying
        // occurs before any subsequent host callback.
        let text = unsafe { copied_utf8(source) }.map_err(|message| ModuleError::Io {
            path: module.path.clone(),
            source: io::Error::new(io::ErrorKind::InvalidData, message),
        })?;
        Ok(SourceFile::new(&module.path, text))
    }
}

/// Releases a buffer returned by THP.
///
/// # Safety
///
/// The fields must be unchanged from a `ThpBuffer` returned by THP. A buffer
/// must be released at most once and not accessed afterward.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thp_buffer_free(buffer: ThpBuffer) {
    if buffer.capacity != 0 {
        // SAFETY: the buffer originated in `ThpBuffer::from_vec`, which leaked
        // exactly this pointer/length/capacity triple.
        unsafe {
            drop(Vec::from_raw_parts(
                buffer.pointer,
                buffer.length,
                buffer.capacity,
            ));
        }
    }
}

pub type ThpLogFn =
    unsafe extern "C" fn(level: u32, message: *const u8, length: usize, user_data: *mut c_void);

#[repr(C)]
pub struct ThpHost {
    pub abi_version: u32,
    pub struct_size: usize,
    pub log: Option<ThpLogFn>,
    pub user_data: *mut c_void,
}

pub type ThpExtensionInitFn = unsafe extern "C" fn(host: *const ThpHost) -> i32;
pub type ThpExtensionShutdownFn = unsafe extern "C" fn();

#[repr(C)]
pub struct ThpExtension {
    pub abi_version: u32,
    pub struct_size: usize,
    pub name: *const c_char,
    pub initialize: Option<ThpExtensionInitFn>,
    pub shutdown: Option<ThpExtensionShutdownFn>,
}

pub type ThpExtensionEntryFn = unsafe extern "C" fn() -> *const ThpExtension;

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use super::{
        Status, THP_ABI_VERSION, ThpBorrowedBuffer, ThpEngineOptions, ThpIo, ThpModuleDescriptor,
        ThpModuleProvider, ThpProjectOptions, thp_buffer_free, thp_engine_free, thp_engine_new,
        thp_engine_new_with_options, thp_engine_prepare_project, thp_engine_run, thp_engine_run_io,
        thp_engine_run_prepared, thp_prepared_project_free,
    };

    #[test]
    fn c_api_runs_and_releases_binary_output() {
        let engine = thp_engine_new(0);
        let path = b"abi.thp";
        let source = b"<?thp\necho \"a\\x00b\";";
        // SAFETY: all pointers reference live Rust byte slices for this call.
        let result = unsafe {
            thp_engine_run(
                engine,
                path.as_ptr(),
                path.len(),
                source.as_ptr(),
                source.len(),
            )
        };
        assert_eq!(result.status, 0);
        // SAFETY: output is a live buffer returned by the function above.
        let output =
            unsafe { std::slice::from_raw_parts(result.output.pointer, result.output.length) };
        assert_eq!(output, b"a\0b");
        // SAFETY: each handle/buffer is released exactly once.
        unsafe {
            thp_buffer_free(result.output);
            thp_buffer_free(result.error);
            thp_engine_free(engine);
        }
    }

    struct IoState {
        input: Vec<u8>,
        position: usize,
        output: Vec<u8>,
    }

    unsafe extern "C" fn read_input(
        user_data: *mut c_void,
        buffer: *mut u8,
        capacity: usize,
        length: *mut usize,
    ) -> i32 {
        // SAFETY: the test passes a live `IoState` and output buffer.
        let state = unsafe { &mut *user_data.cast::<IoState>() };
        let remaining = &state.input[state.position..];
        let count = remaining.len().min(capacity);
        // SAFETY: source and destination are valid and non-overlapping.
        unsafe { std::ptr::copy_nonoverlapping(remaining.as_ptr(), buffer, count) };
        state.position += count;
        // SAFETY: callback contract provides a live length pointer.
        unsafe { *length = count };
        0
    }

    unsafe extern "C" fn write_output(
        user_data: *mut c_void,
        buffer: *const u8,
        length: usize,
    ) -> i32 {
        // SAFETY: the test passes a live state and readable output buffer.
        let state = unsafe { &mut *user_data.cast::<IoState>() };
        let bytes = unsafe { std::slice::from_raw_parts(buffer, length) };
        state.output.extend_from_slice(bytes);
        0
    }

    #[test]
    fn c_api_v1_streams_request_input_and_output() {
        let options = ThpEngineOptions {
            abi_version: THP_ABI_VERSION,
            struct_size: std::mem::size_of::<ThpEngineOptions>(),
            max_instructions: 0,
            max_execution_ms: 0,
            max_heap_bytes: 1024 * 1024,
            max_input_bytes: 3,
            max_input_ms: 1000,
            max_stack_depth: 64,
            max_open_handles: 8,
        };
        // SAFETY: options is a live, complete v1 structure.
        let created = unsafe { thp_engine_new_with_options(&raw const options) };
        assert!(!created.engine.is_null());
        assert_eq!(created.error.length, 0);

        let mut state = IoState {
            input: b"a\0b".to_vec(),
            position: 0,
            output: Vec::new(),
        };
        let io = ThpIo {
            abi_version: THP_ABI_VERSION,
            struct_size: std::mem::size_of::<ThpIo>(),
            input_read: Some(read_input),
            output_write: Some(write_output),
            declared_input_length: 3,
            user_data: (&raw mut state).cast(),
        };
        let path = b"abi-io.thp";
        let source = br#"<?thp
$input = Streams::open("thp:/input", OpenMode::Read);
echo $input->readAll();
"#;
        // SAFETY: every pointer and callback state remains live for this call.
        let result = unsafe {
            thp_engine_run_io(
                created.engine,
                path.as_ptr(),
                path.len(),
                source.as_ptr(),
                source.len(),
                &raw const io,
            )
        };
        assert_eq!(result.status, Status::Success as u32);
        assert_eq!(result.input_bytes, 3);
        assert_eq!(result.output_bytes, 3);
        assert_eq!(state.output, b"a\0b");
        // SAFETY: returned resources are released once.
        unsafe {
            thp_buffer_free(result.error);
            thp_engine_free(created.engine);
        }
    }

    #[test]
    fn c_api_rejects_non_v1_structures() {
        let options = ThpEngineOptions {
            abi_version: 2,
            struct_size: std::mem::size_of::<ThpEngineOptions>(),
            max_instructions: 0,
            max_execution_ms: 0,
            max_heap_bytes: 0,
            max_input_bytes: 0,
            max_input_ms: 0,
            max_stack_depth: 0,
            max_open_handles: 0,
        };
        // SAFETY: options is a live, complete structure, but deliberately has
        // an unsupported version number.
        let created = unsafe { thp_engine_new_with_options(&raw const options) };
        assert!(created.engine.is_null());
        assert_ne!(created.error.length, 0);
        // SAFETY: the returned error buffer is released once.
        unsafe { thp_buffer_free(created.error) };
    }

    struct Fixture {
        id: Vec<u8>,
        path: Vec<u8>,
        namespace: Vec<u8>,
        source: Vec<u8>,
    }

    unsafe extern "C" fn enumerate(
        user_data: *mut c_void,
        index: usize,
        descriptor: *mut ThpModuleDescriptor,
    ) -> i32 {
        if index != 0 {
            return 1;
        }
        // SAFETY: the test passes one live fixture and output descriptor.
        let fixture = unsafe { &*(user_data.cast::<Fixture>()) };
        // SAFETY: the descriptor is live for this callback.
        unsafe {
            *descriptor = ThpModuleDescriptor {
                module_id: borrowed(&fixture.id),
                path: borrowed(&fixture.path),
                expected_namespace: borrowed(&fixture.namespace),
                is_entry: 1,
            };
        }
        0
    }

    unsafe extern "C" fn load(
        user_data: *mut c_void,
        _module_id: ThpBorrowedBuffer,
        source: *mut ThpBorrowedBuffer,
    ) -> i32 {
        // SAFETY: the test passes one live fixture and output buffer.
        let fixture = unsafe { &*(user_data.cast::<Fixture>()) };
        // SAFETY: source is live for this callback.
        unsafe { *source = borrowed(&fixture.source) };
        0
    }

    fn borrowed(bytes: &[u8]) -> ThpBorrowedBuffer {
        ThpBorrowedBuffer {
            pointer: bytes.as_ptr(),
            length: bytes.len(),
        }
    }

    #[test]
    fn c_api_prepares_once_and_executes_repeatedly() {
        let mut fixture = Fixture {
            id: b"App\\Main".to_vec(),
            path: b"main.thp".to_vec(),
            namespace: b"App".to_vec(),
            source: b"<?thp\nnamespace App;\necho \"prepared\";".to_vec(),
        };
        let provider = ThpModuleProvider {
            abi_version: THP_ABI_VERSION,
            struct_size: std::mem::size_of::<ThpModuleProvider>(),
            enumerate: Some(enumerate),
            load: Some(load),
            user_data: std::ptr::from_mut(&mut fixture).cast::<c_void>(),
        };
        let options = ThpProjectOptions {
            abi_version: THP_ABI_VERSION,
            struct_size: std::mem::size_of::<ThpProjectOptions>(),
            project_root: borrowed(b"."),
            entry: borrowed(b"main.thp"),
            target: borrowed(b""),
        };
        let engine = thp_engine_new(0);
        // SAFETY: all ABI structures and callback-owned buffers remain live.
        let prepared =
            unsafe { thp_engine_prepare_project(engine, &raw const provider, &raw const options) };
        assert_eq!(prepared.status, 0);
        assert!(!prepared.project.is_null());
        for _ in 0..2 {
            // SAFETY: both opaque pointers remain live.
            let result = unsafe { thp_engine_run_prepared(engine, prepared.project) };
            assert_eq!(result.status, 0);
            // SAFETY: result buffer is live until freed below.
            let output =
                unsafe { std::slice::from_raw_parts(result.output.pointer, result.output.length) };
            assert_eq!(output, b"prepared");
            // SAFETY: each returned buffer is released once.
            unsafe {
                thp_buffer_free(result.output);
                thp_buffer_free(result.error);
            }
        }
        // SAFETY: every opaque handle and error buffer is released once.
        unsafe {
            thp_buffer_free(prepared.error);
            thp_prepared_project_free(prepared.project);
            thp_engine_free(engine);
        }
    }
}
