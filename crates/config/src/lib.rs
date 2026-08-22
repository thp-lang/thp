//! Project configuration and deterministic lock files for THP.
//!
//! Source configuration is loaded from a required `thp.toml` and an optional
//! `thp.local.toml`. Consumers that need a fast startup path can load
//! `thp.lock`, whose extension payloads remain encoded until requested.

mod diagnostic;
mod lock;
mod model;
mod value;

pub use diagnostic::{Diagnostic, SourceLocation};
pub use lock::{
    LOCK_FILE_NAME, LOCK_VERSION, LockBuild, LockError, LockErrorKind, LockFile, ParsedExtension,
    ParsedLock, ParsedProfile, build_lock, parse_lock,
};
pub use model::{
    AutoloadConfig, ExtensionConfig, ExtensionName, ProjectConfig, ResolvedProfile,
    ResolvedProject, RuntimeConfig, TargetName,
};
pub use value::{ByteSize, Duration, Limit, ParseLimitError};
