//! Project native libraries and checked, call-scoped C value handles.

#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, c_char, c_void};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use libloading::Library;
use thp_bytecode::Program;
use thp_config::{ExtensionConfig, ExtensionName};
use thp_diagnostics::{SourceFile, Span};
use thp_hir::{NativeSignature, PropertyId, Type};
use thp_runtime::Value;
use thp_syntax::{TypeSyntax, TypeSyntaxKind, parse_type_prefix};

pub const THP_ABI_VERSION: u32 = 1;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ThpBorrowedBuffer {
    pub pointer: *const u8,
    pub length: usize,
}

#[repr(C)]
pub struct ThpValue {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ThpCall {
    _private: [u8; 0],
}

pub type ThpLogFn =
    unsafe extern "C" fn(level: u32, message: *const u8, length: usize, user_data: *mut c_void);
pub type ThpNativeCallFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    call: *mut ThpCall,
    arguments: *const *const ThpValue,
    count: usize,
    result: *mut *const ThpValue,
) -> i32;
pub type ThpRegisterFunctionFn =
    unsafe extern "C" fn(user_data: *mut c_void, function: *const ThpNativeFunction) -> i32;

#[repr(C)]
pub struct ThpNativeFunction {
    pub struct_size: usize,
    pub name: ThpBorrowedBuffer,
    pub parameters: *const ThpBorrowedBuffer,
    pub parameter_count: usize,
    pub result: ThpBorrowedBuffer,
    pub callback: Option<ThpNativeCallFn>,
    pub user_data: *mut c_void,
}

#[repr(C)]
pub struct ThpHost {
    pub abi_version: u32,
    pub struct_size: usize,
    pub log: Option<ThpLogFn>,
    pub user_data: *mut c_void,
    pub register_function: Option<ThpRegisterFunctionFn>,
    pub value_kind: Option<extern "C" fn(*mut ThpCall, *const ThpValue) -> i32>,
    pub value_get_int: Option<extern "C" fn(*mut ThpCall, *const ThpValue, *mut i64) -> i32>,
    pub value_get_float: Option<extern "C" fn(*mut ThpCall, *const ThpValue, *mut f64) -> i32>,
    pub value_get_bool: Option<extern "C" fn(*mut ThpCall, *const ThpValue, *mut u8) -> i32>,
    pub value_get_bytes:
        Option<extern "C" fn(*mut ThpCall, *const ThpValue, *mut ThpBorrowedBuffer) -> i32>,
    pub value_count: Option<extern "C" fn(*mut ThpCall, *const ThpValue, *mut usize) -> i32>,
    pub value_item: Option<extern "C" fn(*mut ThpCall, *const ThpValue, usize) -> *const ThpValue>,
    pub value_map_key:
        Option<extern "C" fn(*mut ThpCall, *const ThpValue, usize) -> *const ThpValue>,
    pub value_map_value:
        Option<extern "C" fn(*mut ThpCall, *const ThpValue, usize) -> *const ThpValue>,
    pub value_property:
        Option<extern "C" fn(*mut ThpCall, *const ThpValue, usize) -> *const ThpValue>,
    pub value_new_null: Option<extern "C" fn(*mut ThpCall) -> *const ThpValue>,
    pub value_new_int: Option<extern "C" fn(*mut ThpCall, i64) -> *const ThpValue>,
    pub value_new_float: Option<extern "C" fn(*mut ThpCall, f64) -> *const ThpValue>,
    pub value_new_bool: Option<extern "C" fn(*mut ThpCall, u8) -> *const ThpValue>,
    pub value_new_bytes: Option<extern "C" fn(*mut ThpCall, ThpBorrowedBuffer) -> *const ThpValue>,
    pub value_clone: Option<extern "C" fn(*mut ThpCall, *const ThpValue) -> *const ThpValue>,
    pub value_new_vector: Option<
        extern "C" fn(
            *mut ThpCall,
            ThpBorrowedBuffer,
            *const *const ThpValue,
            usize,
        ) -> *const ThpValue,
    >,
    pub value_new_map: Option<
        extern "C" fn(
            *mut ThpCall,
            ThpBorrowedBuffer,
            ThpBorrowedBuffer,
            *const *const ThpValue,
            *const *const ThpValue,
            usize,
        ) -> *const ThpValue,
    >,
    pub fail: Option<extern "C" fn(*mut ThpCall, ThpBorrowedBuffer) -> i32>,
    pub extension_config: ThpBorrowedBuffer,
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

#[derive(Clone)]
pub struct Registry(Rc<RegistryInner>);

pub struct RegistryScope(Option<Registry>);

impl Drop for RegistryScope {
    fn drop(&mut self) {
        ACTIVE_REGISTRY.with(|active| *active.borrow_mut() = self.0.take());
    }
}

struct RegistryInner {
    libraries: Vec<LoadedLibrary>,
    functions: Vec<LoadedFunction>,
}

struct LoadedLibrary {
    _library: Library,
    shutdown: Option<ThpExtensionShutdownFn>,
}

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown {
            // SAFETY: the library remains loaded until this call returns.
            unsafe { shutdown() };
        }
    }
}

impl Drop for RegistryInner {
    fn drop(&mut self) {
        while let Some(library) = self.libraries.pop() {
            drop(library);
        }
    }
}

#[derive(Clone)]
struct LoadedFunction {
    signature: NativeSignature,
    callback: ThpNativeCallFn,
    user_data: *mut c_void,
}

impl fmt::Debug for Registry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeRegistry")
            .field("functions", &self.0.functions.len())
            .finish()
    }
}

impl Registry {
    pub fn activate(&self) -> RegistryScope {
        let previous = ACTIVE_REGISTRY.with(|active| active.replace(Some(self.clone())));
        RegistryScope(previous)
    }

    pub fn current() -> Option<Self> {
        ACTIVE_REGISTRY.with(|active| active.borrow().clone())
    }

    /// Loads and initializes the selected libraries before compilation.
    ///
    /// # Errors
    /// Returns an error if a library cannot load, has an invalid descriptor, or fails initialization.
    pub fn load(
        root: &Path,
        paths: &BTreeMap<ExtensionName, PathBuf>,
        settings: &BTreeMap<ExtensionName, ExtensionConfig>,
    ) -> Result<Self, String> {
        let mut builder = RegistryBuilder::default();
        for (name, path) in paths {
            let path = if path.is_absolute() {
                path.clone()
            } else {
                root.join(path)
            };
            let bytes = fs::read(&path).map_err(|error| {
                format!("cannot read extension library {}: {error}", path.display())
            })?;
            let digest = blake3::hash(&bytes).to_hex().to_string();
            builder.current_digest = digest;
            settings
                .get(name)
                .map_or("", ExtensionConfig::raw_toml)
                .clone_into(&mut builder.current_config);
            // SAFETY: opening a project-configured native library explicitly opts into its code.
            let library = unsafe { Library::new(&path) }.map_err(|error| {
                format!("cannot load extension library {}: {error}", path.display())
            })?;
            // SAFETY: the symbol is copied while the Library is retained below.
            let entry = unsafe { library.get::<ThpExtensionEntryFn>(b"thp_extension_entry\0") }
                .map_err(|error| {
                    format!("{} has no thp_extension_entry: {error}", path.display())
                })?;
            // SAFETY: the entry point and descriptor obey the public ABI contract.
            let extension = unsafe { entry() };
            if extension.is_null() {
                return Err(format!(
                    "{} returned a null extension descriptor",
                    path.display()
                ));
            }
            // SAFETY: the ABI prefix is readable before the size-versioned tail.
            let abi_version = unsafe { ptr::addr_of!((*extension).abi_version).read() };
            let struct_size = unsafe { ptr::addr_of!((*extension).struct_size).read() };
            if abi_version != THP_ABI_VERSION || struct_size < std::mem::size_of::<ThpExtension>() {
                return Err(format!(
                    "{} has an incompatible extension descriptor",
                    path.display()
                ));
            }
            // SAFETY: the checked size covers the full descriptor while its library is loaded.
            let extension = unsafe { &*extension };
            if extension.name.is_null() {
                return Err(format!(
                    "{} has an incompatible extension descriptor",
                    path.display()
                ));
            }
            // SAFETY: a non-null name in a valid descriptor is a NUL-terminated C string.
            let declared = unsafe { CStr::from_ptr(extension.name) }
                .to_str()
                .map_err(|_| format!("{} has a non-UTF-8 extension name", path.display()))?;
            if declared != name.as_str() {
                return Err(format!(
                    "{} declares extension `{declared}`, expected `{name}`",
                    path.display()
                ));
            }
            let initialize = extension.initialize.ok_or_else(|| {
                format!("{} does not provide an initialize callback", path.display())
            })?;
            let host = host(&mut builder);
            // SAFETY: host and builder remain live for the duration of initialization.
            let status = unsafe { initialize(&raw const host) };
            if status != 0 || builder.error.is_some() {
                if let Some(shutdown) = extension.shutdown {
                    // SAFETY: this library remains loaded through the shutdown callback.
                    unsafe { shutdown() };
                }
                return Err(format!(
                    "extension `{name}` failed to initialize: {}",
                    builder
                        .error
                        .take()
                        .unwrap_or_else(|| format!("status {status}"))
                ));
            }
            builder.libraries.push(LoadedLibrary {
                _library: library,
                shutdown: extension.shutdown,
            });
        }
        Ok(Self(Rc::new(RegistryInner {
            libraries: std::mem::take(&mut builder.libraries),
            functions: std::mem::take(&mut builder.functions),
        })))
    }

    pub fn signatures(&self) -> Vec<NativeSignature> {
        self.0
            .functions
            .iter()
            .map(|function| function.signature.clone())
            .collect()
    }

    /// # Errors
    /// Returns an error when a prepared program used different library content or signatures.
    pub fn verify(&self, expected: &[NativeSignature]) -> Result<(), String> {
        let actual = self.signatures();
        if actual != expected {
            return Err(
                "native library content or signatures differ from the prepared program".to_owned(),
            );
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error for an invalid call, callback failure, or result type mismatch.
    pub fn invoke(
        &self,
        program: &Program,
        index: u32,
        arguments: Vec<Value>,
    ) -> Result<Value, String> {
        let function = self
            .0
            .functions
            .get(index as usize)
            .ok_or_else(|| "native function index is out of bounds".to_owned())?;
        if program.native_functions.get(index as usize) != Some(&function.signature) {
            return Err("native function signature differs from the prepared program".to_owned());
        }
        let mut frame = CallFrame {
            values: Vec::new(),
            error: None,
            program: ptr::from_ref(program),
        };
        let handles = arguments
            .into_iter()
            .map(|value| frame.insert(value))
            .collect::<Vec<_>>();
        if handles.iter().any(|handle| handle.is_null()) {
            return Err("native value handle space exhausted".to_owned());
        }
        let mut result = ptr::null();
        let previous = ACTIVE_CALL.with(|active| active.replace(&raw mut frame));
        // SAFETY: callback belongs to a retained library, and all borrowed handles
        // and the call frame remain live until it returns.
        let status = unsafe {
            (function.callback)(
                function.user_data,
                (&raw mut frame).cast(),
                handles.as_ptr(),
                handles.len(),
                &raw mut result,
            )
        };
        ACTIVE_CALL.with(|active| active.set(previous));
        if status != 0 || frame.error.is_some() {
            return Err(frame
                .error
                .unwrap_or_else(|| format!("native callback returned status {status}")));
        }
        let value = frame
            .get(result)
            .ok_or_else(|| "native callback returned an invalid value handle".to_owned())?
            .clone();
        if !value_matches_type(&value, &function.signature.return_type, program) {
            return Err(format!(
                "native callback `{}` returned {}, expected {}",
                function.signature.name,
                value.type_name(),
                function.signature.return_type
            ));
        }
        Ok(value)
    }
}

#[derive(Default)]
struct RegistryBuilder {
    libraries: Vec<LoadedLibrary>,
    functions: Vec<LoadedFunction>,
    names: BTreeSet<String>,
    current_digest: String,
    current_config: String,
    error: Option<String>,
}

impl Drop for RegistryBuilder {
    fn drop(&mut self) {
        while let Some(library) = self.libraries.pop() {
            drop(library);
        }
    }
}

unsafe extern "C" fn register_function(
    user_data: *mut c_void,
    function: *const ThpNativeFunction,
) -> i32 {
    if user_data.is_null() || function.is_null() {
        return -1;
    }
    // SAFETY: both pointers are supplied only during a synchronous initialize call.
    let builder = unsafe { &mut *user_data.cast::<RegistryBuilder>() };
    // SAFETY: the size field is the first member of the descriptor.
    let struct_size = unsafe { ptr::addr_of!((*function).struct_size).read() };
    if struct_size < std::mem::size_of::<ThpNativeFunction>() {
        builder.error = Some("truncated native function descriptor".to_owned());
        return -1;
    }
    // SAFETY: the checked size covers the descriptor for the duration of registration.
    let function = unsafe { &*function };
    let parsed = (|| {
        let name = unsafe { borrowed_string(function.name) }?.to_owned();
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'\\')
        {
            return Err(format!("invalid native function name `{name}`"));
        }
        if !builder.names.insert(name.clone()) {
            return Err(format!("duplicate native function `{name}`"));
        }
        if function.parameter_count > 256
            || (function.parameter_count > 0 && function.parameters.is_null())
        {
            return Err("invalid native parameter array".to_owned());
        }
        // SAFETY: the descriptor promises a readable parameter array during registration.
        let parameters = if function.parameter_count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(function.parameters, function.parameter_count) }
        }
        .iter()
        .map(|parameter| unsafe { borrowed_string(*parameter) }.and_then(parse_type))
        .collect::<Result<Vec<_>, _>>()?;
        if parameters
            .iter()
            .any(|ty| *ty == Type::Void || *ty == Type::Never)
        {
            return Err("native parameters cannot have void or never type".to_owned());
        }
        let return_type = parse_type(unsafe { borrowed_string(function.result) }?)?;
        let callback = function.callback.ok_or("native function has no callback")?;
        Ok(LoadedFunction {
            signature: NativeSignature {
                name,
                parameters,
                return_type,
                library_digest: builder.current_digest.clone(),
            },
            callback,
            user_data: function.user_data,
        })
    })();
    match parsed {
        Ok(function) => {
            builder.functions.push(function);
            0
        }
        Err(error) => {
            builder.error = Some(error);
            -1
        }
    }
}

fn parse_type(source: &str) -> Result<Type, String> {
    let file = SourceFile::new("<native type>", source);
    let parsed = parse_type_prefix(&file, Span::new(0, source.len()));
    if !parsed.diagnostics.is_empty() || parsed.consumed.end as usize != source.len() {
        return Err(format!("invalid native type `{source}`"));
    }
    convert(
        &parsed
            .ty
            .ok_or_else(|| format!("invalid native type `{source}`"))?,
    )
}

fn convert(ty: &TypeSyntax) -> Result<Type, String> {
    match &ty.kind {
        TypeSyntaxKind::Nullable(inner) => Ok(Type::Union(vec![convert(inner)?, Type::Null])),
        TypeSyntaxKind::Union(types) => Ok(Type::Union(
            types.iter().map(convert).collect::<Result<_, _>>()?,
        )),
        TypeSyntaxKind::Named { name, arguments } => match (name.as_str(), arguments.as_slice()) {
            ("int", []) => Ok(Type::Int),
            ("float", []) => Ok(Type::Float),
            ("bool", []) => Ok(Type::Bool),
            ("string", []) => Ok(Type::String),
            ("null", []) => Ok(Type::Null),
            ("void", []) => Ok(Type::Void),
            ("mixed", []) => Ok(Type::Mixed),
            ("vector", [element]) => Ok(Type::Vector(Box::new(convert(element)?))),
            ("map", [key, value]) => Ok(Type::Map(
                Box::new(convert(key)?),
                Box::new(convert(value)?),
            )),
            (_, []) => Ok(Type::Object(name.clone())),
            _ => Err(format!("unsupported native type `{ty}`")),
        },
    }
}

unsafe fn borrowed_bytes<'a>(buffer: ThpBorrowedBuffer) -> Result<&'a [u8], String> {
    if buffer.length == 0 {
        return Ok(&[]);
    }
    if buffer.pointer.is_null() || buffer.length > isize::MAX as usize {
        return Err("invalid borrowed buffer".to_owned());
    }
    // SAFETY: the C caller promises a readable buffer for this synchronous call.
    Ok(unsafe { std::slice::from_raw_parts(buffer.pointer, buffer.length) })
}

unsafe fn borrowed_string<'a>(buffer: ThpBorrowedBuffer) -> Result<&'a str, String> {
    std::str::from_utf8(unsafe { borrowed_bytes(buffer) }?)
        .map_err(|_| "native text must be UTF-8".to_owned())
}

fn value_matches_type(value: &Value, ty: &Type, program: &Program) -> bool {
    match ty {
        Type::Mixed => true,
        Type::Null | Type::Void => value.type_name() == "null",
        Type::Int => value.as_int().is_some(),
        Type::Float => value.as_float().is_some(),
        Type::Bool => value.as_bool().is_some(),
        Type::String => value.as_bytes().is_some(),
        Type::Vector(element) => value
            .collection_types()
            .is_some_and(|(actual, second)| second.is_none() && element.accepts(&actual)),
        Type::Map(key, element) => {
            value
                .collection_types()
                .is_some_and(|(actual_key, actual_element)| {
                    key.accepts(&actual_key)
                        && actual_element.is_some_and(|actual| element.accepts(&actual))
                })
        }
        Type::Union(types) => types
            .iter()
            .any(|ty| value_matches_type(value, ty, program)),
        Type::Object(_) | Type::Nominal { .. } => value.class_id().is_some_and(|id| {
            program.classes.get(id.0 as usize).is_some_and(|class| {
                let actual = match value.type_arguments() {
                    Some(arguments) if !arguments.is_empty() => Type::Nominal {
                        name: class.name.clone(),
                        arguments: arguments.to_vec(),
                    },
                    _ => Type::Object(class.name.clone()),
                };
                thp_bytecode::type_accepts(program, ty, &actual)
            })
        }),
        Type::Callable(_, _) => value.closure_parts().is_some(),
        Type::Never | Type::Parameter { .. } => false,
    }
}

struct CallFrame {
    values: Vec<(usize, Value)>,
    error: Option<String>,
    program: *const Program,
}

impl CallFrame {
    fn program(&self) -> &Program {
        // SAFETY: the VM program outlives the synchronous native call.
        unsafe { &*self.program }
    }
    fn insert(&mut self, value: Value) -> *const ThpValue {
        let Ok(token) = NEXT_HANDLE.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |token| {
            token.checked_add(1)
        }) else {
            return ptr::null();
        };
        self.values.push((token, value));
        // Opaque tokens never repeat, even when a later call reuses an allocation.
        ptr::without_provenance(token)
    }

    fn get(&self, handle: *const ThpValue) -> Option<&Value> {
        self.values
            .iter()
            .find(|(token, _)| *token == handle.addr())
            .map(|(_, value)| value)
    }
}

static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(0x1_0000);

thread_local! {
    static ACTIVE_CALL: Cell<*mut CallFrame> = const { Cell::new(ptr::null_mut()) };
    static ACTIVE_REGISTRY: RefCell<Option<Registry>> = const { RefCell::new(None) };
}

fn with_frame<T>(call: *mut ThpCall, f: impl FnOnce(&mut CallFrame) -> T) -> Option<T> {
    ACTIVE_CALL.with(|active| {
        let frame = active.get();
        if frame.is_null() || frame.cast::<ThpCall>() != call {
            return None;
        }
        // SAFETY: ACTIVE_CALL is set only during synchronous invocation on this thread.
        Some(f(unsafe { &mut *frame }))
    })
}

fn host(builder: &mut RegistryBuilder) -> ThpHost {
    ThpHost {
        abi_version: THP_ABI_VERSION,
        struct_size: std::mem::size_of::<ThpHost>(),
        log: None,
        user_data: ptr::from_mut(builder).cast(),
        register_function: Some(register_function),
        value_kind: Some(value_kind),
        value_get_int: Some(value_get_int),
        value_get_float: Some(value_get_float),
        value_get_bool: Some(value_get_bool),
        value_get_bytes: Some(value_get_bytes),
        value_count: Some(value_count),
        value_item: Some(value_item),
        value_map_key: Some(value_map_key),
        value_map_value: Some(value_map_value),
        value_property: Some(value_property),
        value_new_null: Some(value_new_null),
        value_new_int: Some(value_new_int),
        value_new_float: Some(value_new_float),
        value_new_bool: Some(value_new_bool),
        value_new_bytes: Some(value_new_bytes),
        value_clone: Some(value_clone),
        value_new_vector: Some(value_new_vector),
        value_new_map: Some(value_new_map),
        fail: Some(fail),
        extension_config: ThpBorrowedBuffer {
            pointer: builder.current_config.as_ptr(),
            length: builder.current_config.len(),
        },
    }
}

extern "C" fn value_kind(call: *mut ThpCall, handle: *const ThpValue) -> i32 {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .map_or(-1, |value| match value.type_name() {
                "null" => 0,
                "int" => 1,
                "float" => 2,
                "bool" => 3,
                "string" => 4,
                "vector" => 5,
                "map" => 6,
                "object" => 7,
                "callable" => 8,
                "stream" => 9,
                "exception" => 10,
                _ => -1,
            })
    })
    .unwrap_or(-1)
}

macro_rules! scalar_getter {
    ($name:ident, $type:ty, $method:ident) => {
        extern "C" fn $name(call: *mut ThpCall, handle: *const ThpValue, out: *mut $type) -> i32 {
            if out.is_null() {
                return -1;
            }
            with_frame(call, |frame| {
                let Some(value) = frame.get(handle).and_then(Value::$method) else {
                    return -1;
                };
                // SAFETY: the C caller provides a writable output slot.
                unsafe {
                    *out = value as $type;
                }
                0
            })
            .unwrap_or(-1)
        }
    };
}

scalar_getter!(value_get_int, i64, as_int);
scalar_getter!(value_get_float, f64, as_float);
scalar_getter!(value_get_bool, u8, as_bool);

extern "C" fn value_get_bytes(
    call: *mut ThpCall,
    handle: *const ThpValue,
    out: *mut ThpBorrowedBuffer,
) -> i32 {
    if out.is_null() {
        return -1;
    }
    with_frame(call, |frame| {
        let Some(bytes) = frame.get(handle).and_then(Value::as_bytes) else {
            return -1;
        };
        // SAFETY: the C caller provides a writable output slot; bytes live through this call.
        unsafe {
            *out = ThpBorrowedBuffer {
                pointer: bytes.as_ptr(),
                length: bytes.len(),
            };
        }
        0
    })
    .unwrap_or(-1)
}

extern "C" fn value_count(call: *mut ThpCall, handle: *const ThpValue, out: *mut usize) -> i32 {
    if out.is_null() {
        return -1;
    }
    with_frame(call, |frame| {
        let Some(value) = frame.get(handle).and_then(Value::count) else {
            return -1;
        };
        // SAFETY: the C caller provides a writable output slot.
        unsafe {
            *out = value;
        }
        0
    })
    .unwrap_or(-1)
}

extern "C" fn value_item(
    call: *mut ThpCall,
    handle: *const ThpValue,
    index: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .and_then(Value::vector_values)
            .and_then(|values| values.get(index))
            .cloned()
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_map_key(
    call: *mut ThpCall,
    handle: *const ThpValue,
    index: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .and_then(Value::map_entries)
            .and_then(|entries| entries.get(index))
            .map(|pair| pair.0.clone())
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_map_value(
    call: *mut ThpCall,
    handle: *const ThpValue,
    index: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .and_then(Value::map_entries)
            .and_then(|entries| entries.get(index))
            .map(|pair| pair.1.clone())
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_property(
    call: *mut ThpCall,
    handle: *const ThpValue,
    index: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .and_then(|value| value.property(PropertyId(u32::try_from(index).ok()?)).ok())
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_new_null(call: *mut ThpCall) -> *const ThpValue {
    with_frame(call, |frame| frame.insert(Value::NULL)).unwrap_or(ptr::null())
}
extern "C" fn value_new_int(call: *mut ThpCall, value: i64) -> *const ThpValue {
    with_frame(call, |frame| frame.insert(Value::integer(value))).unwrap_or(ptr::null())
}
extern "C" fn value_new_float(call: *mut ThpCall, value: f64) -> *const ThpValue {
    with_frame(call, |frame| frame.insert(Value::float(value))).unwrap_or(ptr::null())
}
extern "C" fn value_new_bool(call: *mut ThpCall, value: u8) -> *const ThpValue {
    with_frame(call, |frame| frame.insert(Value::bool(value != 0))).unwrap_or(ptr::null())
}
extern "C" fn value_new_bytes(call: *mut ThpCall, bytes: ThpBorrowedBuffer) -> *const ThpValue {
    with_frame(call, |frame| {
        // SAFETY: the C caller promises the buffer is readable during this call.
        unsafe { borrowed_bytes(bytes) }
            .ok()
            .and_then(|bytes| Value::try_bytes(bytes).ok())
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}
extern "C" fn value_clone(call: *mut ThpCall, handle: *const ThpValue) -> *const ThpValue {
    with_frame(call, |frame| {
        frame
            .get(handle)
            .cloned()
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_new_vector(
    call: *mut ThpCall,
    element: ThpBorrowedBuffer,
    handles: *const *const ThpValue,
    count: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        if count > 1_000_000 || (count > 0 && handles.is_null()) {
            return ptr::null();
        }
        // SAFETY: the C caller promises a readable handle array during this call.
        let handles = if count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(handles, count) }
        };
        let Some(ty) = (unsafe { borrowed_string(element) })
            .ok()
            .and_then(|source| parse_type(source).ok())
        else {
            return ptr::null();
        };
        let values = handles
            .iter()
            .map(|handle| frame.get(*handle).cloned())
            .collect::<Option<Vec<_>>>();
        let Some(values) = values else {
            return ptr::null();
        };
        if values
            .iter()
            .any(|value| !value_matches_type(value, &ty, frame.program()))
        {
            return ptr::null();
        }
        Value::try_vector(ty, values).map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn value_new_map(
    call: *mut ThpCall,
    key_type: ThpBorrowedBuffer,
    value_type: ThpBorrowedBuffer,
    keys: *const *const ThpValue,
    values: *const *const ThpValue,
    count: usize,
) -> *const ThpValue {
    with_frame(call, |frame| {
        if count > 1_000_000 || (count > 0 && (keys.is_null() || values.is_null())) {
            return ptr::null();
        }
        // SAFETY: the C caller promises readable handle arrays during this call.
        let keys = if count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(keys, count) }
        };
        let values = if count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(values, count) }
        };
        let Some(key_type) = (unsafe { borrowed_string(key_type) })
            .ok()
            .and_then(|source| parse_type(source).ok())
        else {
            return ptr::null();
        };
        let Some(value_type) = (unsafe { borrowed_string(value_type) })
            .ok()
            .and_then(|source| parse_type(source).ok())
        else {
            return ptr::null();
        };
        let entries = keys
            .iter()
            .zip(values)
            .map(|(key, value)| Some((frame.get(*key)?.clone(), frame.get(*value)?.clone())))
            .collect::<Option<Vec<_>>>();
        let Some(entries) = entries else {
            return ptr::null();
        };
        if entries.iter().any(|(key, value)| {
            !value_matches_type(key, &key_type, frame.program())
                || !value_matches_type(value, &value_type, frame.program())
        }) {
            return ptr::null();
        }
        Value::try_map(key_type, value_type, entries)
            .map_or(ptr::null(), |value| frame.insert(value))
    })
    .unwrap_or(ptr::null())
}

extern "C" fn fail(call: *mut ThpCall, message: ThpBorrowedBuffer) -> i32 {
    with_frame(call, |frame| {
        // SAFETY: the C caller promises the message is readable during this call.
        match unsafe { borrowed_bytes(message) } {
            Ok(bytes) => {
                frame.error = Some(String::from_utf8_lossy(bytes).into_owned());
                0
            }
            Err(_) => -1,
        }
    })
    .unwrap_or(-1)
}
