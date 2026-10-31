use std::io::{BufRead, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, mpsc};

use thp_hir::FileInfoBuiltin;

use super::{RuntimeErrorKind, StreamStorage, Value, ensure_stream_open, stream_io_error};

pub enum StreamFileOperation {
    Read(usize),
    ReadAll(Option<usize>),
    ReadLine,
    SeekLine(usize),
    Eof,
    Seek { offset: i64, origin: i64 },
    Write(Vec<u8>),
    Flush,
}

pub enum StreamFileResult {
    Bytes(Vec<u8>),
    Line(Option<Vec<u8>>),
    Written(usize),
    Flushed,
    Bool(bool),
    Position(usize),
}

pub enum StreamFileStart {
    Busy,
    QueueFull,
    Pending(StreamFilePending),
}

pub struct StreamFilePending {
    owner: Value,
    receiver: mpsc::Receiver<Result<JobResult, RuntimeErrorKind>>,
    completed: Option<Result<StreamFileResult, RuntimeErrorKind>>,
}

pub enum OpenFileStart {
    QueueFull,
    Pending(OpenFilePending),
}

pub struct OpenFilePending {
    receiver: mpsc::Receiver<std::io::Result<std::fs::File>>,
    completed: Option<std::io::Result<std::fs::File>>,
}

pub enum FileInfoStart {
    QueueFull,
    Pending(FileInfoPending),
}

pub struct FileInfoPending {
    receiver: mpsc::Receiver<FileInfoValue>,
    completed: Option<Result<FileInfoValue, RuntimeErrorKind>>,
}

pub enum DirectoryStart {
    QueueFull,
    Pending(DirectoryPending),
}

pub struct DirectoryPending {
    receiver: mpsc::Receiver<Result<Vec<Vec<u8>>, DirectoryReadError>>,
    completed: Option<Result<Vec<Vec<u8>>, DirectoryReadError>>,
}

pub enum RecursiveDirectoryStart {
    QueueFull,
    Pending(RecursiveDirectoryPending),
}

pub struct RecursiveDirectoryPending {
    receiver: mpsc::Receiver<Result<Option<Vec<Vec<u8>>>, DirectoryReadError>>,
    completed: Option<Result<Option<Vec<Vec<u8>>>, DirectoryReadError>>,
}

pub enum DirectoryReadError {
    Open(std::io::Error),
    Entry(std::io::Error),
    NonUtf8,
    WorkerStopped,
}

pub enum FileInfoValue {
    Bytes(Vec<u8>),
    Integer(i64),
    Bool(bool),
    False,
}

#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct OpenFileFlags {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
    pub exclusive: bool,
}

enum Job {
    Stream {
        file: Option<std::fs::File>,
        operation: Work,
        response: mpsc::Sender<Result<JobResult, RuntimeErrorKind>>,
    },
    Open {
        path: Vec<u8>,
        flags: OpenFileFlags,
        response: mpsc::Sender<std::io::Result<std::fs::File>>,
    },
    Info {
        path: Vec<u8>,
        operation: FileInfoBuiltin,
        response: mpsc::Sender<FileInfoValue>,
    },
    Directory {
        path: Vec<u8>,
        response: mpsc::Sender<Result<Vec<Vec<u8>>, DirectoryReadError>>,
    },
    RecursiveDirectory {
        path: Vec<u8>,
        follow: bool,
        response: mpsc::Sender<Result<Option<Vec<Vec<u8>>>, DirectoryReadError>>,
    },
}

enum Work {
    Read {
        cursor: usize,
        length: usize,
    },
    ReadAll {
        cursor: usize,
        limit: Option<usize>,
    },
    ReadLine {
        cursor: usize,
    },
    SeekLine {
        line: usize,
    },
    Eof {
        cursor: usize,
    },
    Seek {
        cursor: usize,
        offset: i64,
        origin: i64,
    },
    Write {
        cursor: usize,
        bytes: Vec<u8>,
        append: bool,
    },
    SpillWrite {
        existing: Vec<u8>,
        cursor: usize,
        bytes: Vec<u8>,
    },
    Flush,
}

struct JobResult {
    result: StreamFileResult,
    cursor: Option<usize>,
    replacement: Option<std::fs::File>,
}

fn pool() -> Result<&'static mpsc::SyncSender<Job>, RuntimeErrorKind> {
    static POOL: OnceLock<Result<mpsc::SyncSender<Job>, String>> = OnceLock::new();
    POOL.get_or_init(|| {
        let (sender, receiver) = mpsc::sync_channel::<Job>(64);
        let receiver = Arc::new(Mutex::new(receiver));
        for index in 0..4 {
            let receiver = Arc::clone(&receiver);
            std::thread::Builder::new()
                .name(format!("thp-file-{index}"))
                .spawn(move || {
                    loop {
                        let Ok(job) = receiver.lock().expect("file queue mutex poisoned").recv()
                        else {
                            break;
                        };
                        match job {
                            Job::Stream {
                                file,
                                operation,
                                response,
                            } => {
                                let _ = response.send(process(file, operation));
                            }
                            Job::Open {
                                path,
                                flags,
                                response,
                            } => {
                                let _ = response.send(open_file(path_from_bytes(path), flags));
                            }
                            Job::Info {
                                path,
                                operation,
                                response,
                            } => {
                                let _ = response
                                    .send(file_info_host(&path_from_bytes(path), operation));
                            }
                            Job::Directory { path, response } => {
                                let _ = response.send(read_directory(path_from_bytes(path)));
                            }
                            Job::RecursiveDirectory {
                                path,
                                follow,
                                response,
                            } => {
                                let path = path_from_bytes(path);
                                let result = if path.is_dir() && (follow || !path.is_symlink()) {
                                    read_directory(path).map(Some)
                                } else {
                                    Ok(None)
                                };
                                let _ = response.send(result);
                            }
                        }
                    }
                })
                .map_err(|error| error.to_string())?;
        }
        Ok(sender)
    })
    .as_ref()
    .map_err(|error| RuntimeErrorKind::Io(format!("cannot start file workers: {error}")))
}

fn open_file(path: PathBuf, flags: OpenFileFlags) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options
        .read(flags.read)
        .write(flags.write && !flags.append)
        .append(flags.append)
        .truncate(flags.truncate)
        .create(flags.create && !flags.exclusive)
        .create_new(flags.exclusive);
    let file = options.open(path)?;
    if file.metadata()?.is_file() {
        Ok(file)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a regular file",
        ))
    }
}

#[allow(clippy::unnecessary_wraps)]
fn path_bytes(path: PathBuf) -> Result<Vec<u8>, RuntimeErrorKind> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(path.into_os_string().into_vec())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        Ok(path
            .as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect())
    }
    #[cfg(not(any(unix, windows)))]
    {
        path.into_os_string()
            .into_string()
            .map(String::into_bytes)
            .map_err(|_| RuntimeErrorKind::Io("filesystem path must be UTF-8".to_owned()))
    }
}

fn path_from_bytes(bytes: Vec<u8>) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(bytes).into()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let wide = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        std::ffi::OsString::from_wide(&wide).into()
    }
    #[cfg(not(any(unix, windows)))]
    {
        std::ffi::OsString::from(String::from_utf8(bytes).expect("encoded UTF-8 path")).into()
    }
}

/// Queues an open without transferring any request-local values to a worker.
///
/// # Errors
///
/// Returns a worker-pool failure.
pub fn start_open_file(
    path: PathBuf,
    flags: OpenFileFlags,
) -> Result<OpenFileStart, RuntimeErrorKind> {
    let (response, receiver) = mpsc::channel();
    match pool()?.try_send(Job::Open {
        path: path_bytes(path)?,
        flags,
        response,
    }) {
        Ok(()) => Ok(OpenFileStart::Pending(OpenFilePending {
            receiver,
            completed: None,
        })),
        Err(mpsc::TrySendError::Full(_)) => Ok(OpenFileStart::QueueFull),
        Err(mpsc::TrySendError::Disconnected(_)) => {
            Err(RuntimeErrorKind::Io("file workers stopped".to_owned()))
        }
    }
}

impl OpenFilePending {
    pub fn poll(&mut self) -> bool {
        if self.completed.is_some() {
            return true;
        }
        self.completed = match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(std::io::Error::other("file worker stopped")))
            }
        };
        self.completed.is_some()
    }

    pub fn take_result(&mut self) -> Option<std::io::Result<std::fs::File>> {
        self.completed.take()
    }
}

/// Queues one filesystem metadata query.
///
/// # Errors
///
/// Returns a worker-pool failure.
pub fn start_file_info(
    path: PathBuf,
    operation: FileInfoBuiltin,
) -> Result<FileInfoStart, RuntimeErrorKind> {
    let (response, receiver) = mpsc::channel();
    match pool()?.try_send(Job::Info {
        path: path_bytes(path)?,
        operation,
        response,
    }) {
        Ok(()) => Ok(FileInfoStart::Pending(FileInfoPending {
            receiver,
            completed: None,
        })),
        Err(mpsc::TrySendError::Full(_)) => Ok(FileInfoStart::QueueFull),
        Err(mpsc::TrySendError::Disconnected(_)) => {
            Err(RuntimeErrorKind::Io("file workers stopped".to_owned()))
        }
    }
}

impl FileInfoPending {
    pub fn poll(&mut self) -> bool {
        if self.completed.is_some() {
            return true;
        }
        self.completed = match self.receiver.try_recv() {
            Ok(result) => Some(Ok(result)),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(RuntimeErrorKind::Io("file worker stopped".to_owned())))
            }
        };
        self.completed.is_some()
    }

    pub fn take_result(&mut self) -> Option<Result<FileInfoValue, RuntimeErrorKind>> {
        self.completed.take()
    }
}

/// Queues a directory read and returns only UTF-8 entry names.
///
/// # Errors
///
/// Returns a worker-pool failure.
pub fn start_directory_read(path: PathBuf) -> Result<DirectoryStart, RuntimeErrorKind> {
    let (response, receiver) = mpsc::channel();
    match pool()?.try_send(Job::Directory {
        path: path_bytes(path)?,
        response,
    }) {
        Ok(()) => Ok(DirectoryStart::Pending(DirectoryPending {
            receiver,
            completed: None,
        })),
        Err(mpsc::TrySendError::Full(_)) => Ok(DirectoryStart::QueueFull),
        Err(mpsc::TrySendError::Disconnected(_)) => {
            Err(RuntimeErrorKind::Io("file workers stopped".to_owned()))
        }
    }
}

fn read_directory(path: PathBuf) -> Result<Vec<Vec<u8>>, DirectoryReadError> {
    std::fs::read_dir(path)
        .map_err(DirectoryReadError::Open)?
        .map(|entry| {
            entry
                .map_err(DirectoryReadError::Entry)?
                .file_name()
                .into_string()
                .map(String::into_bytes)
                .map_err(|_| DirectoryReadError::NonUtf8)
        })
        .collect()
}

impl DirectoryPending {
    pub fn poll(&mut self) -> bool {
        if self.completed.is_some() {
            return true;
        }
        self.completed = match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(DirectoryReadError::WorkerStopped)),
        };
        self.completed.is_some()
    }

    pub fn take_result(&mut self) -> Option<Result<Vec<Vec<u8>>, DirectoryReadError>> {
        self.completed.take()
    }
}

/// Queues the child check and directory read used by recursive iteration.
///
/// # Errors
///
/// Returns a worker-pool failure.
pub fn start_recursive_directory_read(
    path: PathBuf,
    follow: bool,
) -> Result<RecursiveDirectoryStart, RuntimeErrorKind> {
    let (response, receiver) = mpsc::channel();
    match pool()?.try_send(Job::RecursiveDirectory {
        path: path_bytes(path)?,
        follow,
        response,
    }) {
        Ok(()) => Ok(RecursiveDirectoryStart::Pending(
            RecursiveDirectoryPending {
                receiver,
                completed: None,
            },
        )),
        Err(mpsc::TrySendError::Full(_)) => Ok(RecursiveDirectoryStart::QueueFull),
        Err(mpsc::TrySendError::Disconnected(_)) => {
            Err(RuntimeErrorKind::Io("file workers stopped".to_owned()))
        }
    }
}

impl RecursiveDirectoryPending {
    pub fn poll(&mut self) -> bool {
        if self.completed.is_some() {
            return true;
        }
        self.completed = match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(DirectoryReadError::WorkerStopped)),
        };
        self.completed.is_some()
    }

    pub fn take_result(&mut self) -> Option<Result<Option<Vec<Vec<u8>>>, DirectoryReadError>> {
        self.completed.take()
    }
}

#[must_use]
pub fn file_info_uses_host(operation: FileInfoBuiltin) -> bool {
    !matches!(
        operation,
        FileInfoBuiltin::Construct
            | FileInfoBuiltin::Pathname
            | FileInfoBuiltin::ToString
            | FileInfoBuiltin::Path
            | FileInfoBuiltin::Filename
            | FileInfoBuiltin::Basename
            | FileInfoBuiltin::Extension
    )
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn file_info_host(path: &std::path::Path, operation: FileInfoBuiltin) -> FileInfoValue {
    use FileInfoBuiltin as Op;
    match operation {
        Op::RealPath => std::fs::canonicalize(path)
            .ok()
            .and_then(|path| path.to_str().map(str::to_owned))
            .map_or(FileInfoValue::False, |value| {
                FileInfoValue::Bytes(value.into_bytes())
            }),
        Op::LinkTarget => std::fs::read_link(path)
            .ok()
            .and_then(|path| path.to_str().map(str::to_owned))
            .map_or(FileInfoValue::False, |value| {
                FileInfoValue::Bytes(value.into_bytes())
            }),
        Op::Type => std::fs::symlink_metadata(path)
            .ok()
            .map_or(FileInfoValue::False, |metadata| {
                let ty = metadata.file_type();
                FileInfoValue::Bytes(if ty.is_symlink() {
                    b"link".to_vec()
                } else if ty.is_dir() {
                    b"dir".to_vec()
                } else if ty.is_file() {
                    b"file".to_vec()
                } else {
                    b"unknown".to_vec()
                })
            }),
        Op::IsFile => FileInfoValue::Bool(path.is_file()),
        Op::IsDir => FileInfoValue::Bool(path.is_dir()),
        Op::IsLink => FileInfoValue::Bool(path.is_symlink()),
        Op::IsReadable => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                FileInfoValue::Bool(
                    std::fs::metadata(path)
                        .is_ok_and(|metadata| metadata.permissions().mode() & 0o444 != 0),
                )
            }
            #[cfg(not(unix))]
            {
                FileInfoValue::Bool(std::fs::metadata(path).is_ok())
            }
        }
        Op::IsWritable => FileInfoValue::Bool(
            std::fs::metadata(path).is_ok_and(|metadata| !metadata.permissions().readonly()),
        ),
        Op::IsExecutable => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                FileInfoValue::Bool(
                    std::fs::metadata(path)
                        .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0),
                )
            }
            #[cfg(not(unix))]
            {
                FileInfoValue::Bool(false)
            }
        }
        Op::Size
        | Op::ATime
        | Op::MTime
        | Op::CTime
        | Op::Perms
        | Op::Owner
        | Op::Group
        | Op::Inode => std::fs::metadata(path)
            .ok()
            .and_then(|metadata| match operation {
                Op::Size => i64::try_from(metadata.len()).ok(),
                Op::ATime => metadata.accessed().ok().and_then(unix_time),
                Op::MTime => metadata.modified().ok().and_then(unix_time),
                #[cfg(unix)]
                _ => {
                    use std::os::unix::fs::MetadataExt;
                    match operation {
                        Op::CTime => Some(metadata.ctime()),
                        Op::Perms => Some(i64::from(metadata.mode())),
                        Op::Owner => Some(i64::from(metadata.uid())),
                        Op::Group => Some(i64::from(metadata.gid())),
                        Op::Inode => i64::try_from(metadata.ino()).ok(),
                        _ => None,
                    }
                }
                #[cfg(not(unix))]
                _ => None,
            })
            .map_or(FileInfoValue::False, FileInfoValue::Integer),
        _ => unreachable!("only host metadata operations are submitted"),
    }
}

fn unix_time(time: std::time::SystemTime) -> Option<i64> {
    time.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
}

#[allow(clippy::too_many_lines)]
fn process(file: Option<std::fs::File>, operation: Work) -> Result<JobResult, RuntimeErrorKind> {
    let mut file = match file {
        Some(file) => file,
        None => tempfile::tempfile().map_err(stream_io_error)?,
    };
    let mut replacement = None;
    let (result, cursor) = match operation {
        Work::Read { cursor, length } => {
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            let mut bytes = Vec::new();
            (&mut file)
                .take(length as u64)
                .read_to_end(&mut bytes)
                .map_err(stream_io_error)?;
            let end = cursor.saturating_add(bytes.len());
            (StreamFileResult::Bytes(bytes), Some(end))
        }
        Work::ReadAll { cursor, limit } => {
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            let mut bytes = Vec::new();
            (&mut file)
                .take(limit.map_or(u64::MAX, |limit| (limit as u64).saturating_add(1)))
                .read_to_end(&mut bytes)
                .map_err(stream_io_error)?;
            if limit.is_some_and(|limit| bytes.len() > limit) {
                return Err(RuntimeErrorKind::Io(
                    "stream read limit exceeded".to_owned(),
                ));
            }
            let end = cursor.saturating_add(bytes.len());
            (StreamFileResult::Bytes(bytes), Some(end))
        }
        Work::ReadLine { cursor } => {
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            let mut bytes = Vec::new();
            let read = std::io::BufReader::new(&mut file)
                .read_until(b'\n', &mut bytes)
                .map_err(stream_io_error)?;
            (
                StreamFileResult::Line((read > 0).then_some(bytes)),
                Some(cursor.saturating_add(read)),
            )
        }
        Work::SeekLine { line } => return seek_line(&mut file, line),
        Work::Eof { cursor } => {
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            let mut byte = [0];
            let eof = file.read(&mut byte).map_err(stream_io_error)? == 0;
            (StreamFileResult::Bool(eof), None)
        }
        Work::Seek {
            cursor,
            offset,
            origin,
        } => return seek_file(&file, cursor, offset, origin),
        Work::Write {
            cursor,
            bytes,
            append,
        } => {
            let cursor = if append {
                usize::try_from(file.metadata().map_err(stream_io_error)?.len())
                    .map_err(|_| RuntimeErrorKind::Io("file is too large".to_owned()))?
            } else {
                cursor
            };
            if (cursor as u64) > file.metadata().map_err(stream_io_error)?.len() {
                file.set_len(cursor as u64).map_err(stream_io_error)?;
            }
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            file.write_all(&bytes).map_err(stream_io_error)?;
            let end = cursor
                .checked_add(bytes.len())
                .ok_or_else(|| RuntimeErrorKind::Io("stream position overflow".to_owned()))?;
            (StreamFileResult::Written(bytes.len()), Some(end))
        }
        Work::SpillWrite {
            existing,
            cursor,
            bytes,
        } => {
            file.write_all(&existing).map_err(stream_io_error)?;
            if cursor > existing.len() {
                file.set_len(cursor as u64).map_err(stream_io_error)?;
            }
            file.seek(SeekFrom::Start(cursor as u64))
                .map_err(stream_io_error)?;
            file.write_all(&bytes).map_err(stream_io_error)?;
            let end = cursor
                .checked_add(bytes.len())
                .ok_or_else(|| RuntimeErrorKind::Io("stream position overflow".to_owned()))?;
            replacement = Some(file.try_clone().map_err(stream_io_error)?);
            (StreamFileResult::Written(bytes.len()), Some(end))
        }
        Work::Flush => {
            file.flush().map_err(stream_io_error)?;
            (StreamFileResult::Flushed, None)
        }
    };
    Ok(JobResult {
        result,
        cursor,
        replacement,
    })
}

fn seek_line(file: &mut std::fs::File, line: usize) -> Result<JobResult, RuntimeErrorKind> {
    file.seek(SeekFrom::Start(0)).map_err(stream_io_error)?;
    let mut reader = std::io::BufReader::new(file);
    let mut bytes = Vec::new();
    let mut cursor = 0usize;
    for _ in 0..line {
        bytes.clear();
        let read = reader
            .read_until(b'\n', &mut bytes)
            .map_err(stream_io_error)?;
        cursor = cursor.saturating_add(read);
        if read == 0 {
            return Ok(JobResult {
                result: StreamFileResult::Line(None),
                cursor: Some(cursor),
                replacement: None,
            });
        }
    }
    bytes.clear();
    let read = reader
        .read_until(b'\n', &mut bytes)
        .map_err(stream_io_error)?;
    cursor = cursor.saturating_add(read);
    Ok(JobResult {
        result: StreamFileResult::Line((read > 0).then_some(bytes)),
        cursor: Some(cursor),
        replacement: None,
    })
}

fn seek_file(
    file: &std::fs::File,
    cursor: usize,
    offset: i64,
    origin: i64,
) -> Result<JobResult, RuntimeErrorKind> {
    let base = match origin {
        0 => 0,
        1 => cursor,
        2 => usize::try_from(file.metadata().map_err(stream_io_error)?.len())
            .map_err(|_| RuntimeErrorKind::Io("file is too large".to_owned()))?,
        _ => return Err(RuntimeErrorKind::Io("invalid seek origin".to_owned())),
    };
    let position = (base as i128) + i128::from(offset);
    if position < 0 {
        return Err(RuntimeErrorKind::Bounds(
            "stream position must not be negative".to_owned(),
        ));
    }
    if position > i128::from(i64::MAX) {
        return Err(RuntimeErrorKind::Io("stream position overflow".to_owned()));
    }
    let position = usize::try_from(position)
        .map_err(|_| RuntimeErrorKind::Io("stream position overflow".to_owned()))?;
    Ok(JobResult {
        result: StreamFileResult::Position(position),
        cursor: Some(position),
        replacement: None,
    })
}

impl Value {
    /// Starts a file operation while retaining the stream's shared cursor.
    ///
    /// # Errors
    ///
    /// Returns stream, allocation, or worker-pool failures.
    #[allow(clippy::too_many_lines)]
    pub fn start_stream_file_operation(
        &self,
        operation: StreamFileOperation,
    ) -> Result<Option<StreamFileStart>, RuntimeErrorKind> {
        let cell = self.stream_state()?;
        let mut state = cell.borrow_mut();
        if state.busy {
            return Ok(Some(StreamFileStart::Busy));
        }
        ensure_stream_open(&state)?;
        let reading = matches!(
            operation,
            StreamFileOperation::Read(_)
                | StreamFileOperation::ReadAll(_)
                | StreamFileOperation::ReadLine
                | StreamFileOperation::SeekLine(_)
                | StreamFileOperation::Eof
        );
        let writing = matches!(
            operation,
            StreamFileOperation::Write(_) | StreamFileOperation::Flush
        );
        if reading && !state.has(super::STREAM_READ) {
            return Err(RuntimeErrorKind::Io(
                "stream does not support reading".to_owned(),
            ));
        }
        if writing && !state.has(super::STREAM_WRITE) {
            return Err(RuntimeErrorKind::Io(
                "stream does not support writing".to_owned(),
            ));
        }
        if matches!(operation, StreamFileOperation::Seek { .. }) && !state.has(super::STREAM_SEEK) {
            return Err(RuntimeErrorKind::Io(
                "stream does not support seeking".to_owned(),
            ));
        }
        let (file, work) = match (&state.storage, operation) {
            (StreamStorage::File { file }, StreamFileOperation::Read(length)) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::Read {
                    cursor: state.cursor,
                    length,
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::ReadAll(limit)) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::ReadAll {
                    cursor: state.cursor,
                    limit,
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::ReadLine) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::ReadLine {
                    cursor: state.cursor,
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::SeekLine(line)) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::SeekLine { line },
            ),
            (StreamStorage::File { file }, StreamFileOperation::Eof) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::Eof {
                    cursor: state.cursor,
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::Seek { offset, origin }) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::Seek {
                    cursor: state.cursor,
                    offset,
                    origin,
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::Write(bytes)) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::Write {
                    cursor: state.cursor,
                    bytes,
                    append: state.has(super::STREAM_APPEND),
                },
            ),
            (StreamStorage::File { file }, StreamFileOperation::Flush) => (
                Some(file.try_clone().map_err(stream_io_error)?),
                Work::Flush,
            ),
            (StreamStorage::Memory(existing), StreamFileOperation::Write(bytes))
                if state
                    .spill_threshold
                    .is_some_and(|limit| state.cursor.saturating_add(bytes.len()) > limit) =>
            {
                (
                    None,
                    Work::SpillWrite {
                        existing: existing.clone(),
                        cursor: state.cursor,
                        bytes,
                    },
                )
            }
            _ => return Ok(None),
        };
        let (response, receiver) = mpsc::channel();
        match pool()?.try_send(Job::Stream {
            file,
            operation: work,
            response,
        }) {
            Ok(()) => {}
            Err(mpsc::TrySendError::Full(_)) => return Ok(Some(StreamFileStart::QueueFull)),
            Err(mpsc::TrySendError::Disconnected(_)) => {
                return Err(RuntimeErrorKind::Io("file workers stopped".to_owned()));
            }
        }
        state.busy = true;
        Ok(Some(StreamFileStart::Pending(StreamFilePending {
            owner: self.clone(),
            receiver,
            completed: None,
        })))
    }

    /// Reports whether an aliased stream has an outstanding file job.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not a stream.
    pub fn stream_file_busy(&self) -> Result<bool, RuntimeErrorKind> {
        Ok(self.stream_state()?.borrow().busy)
    }
}

impl StreamFilePending {
    /// Applies a completed worker result to the request-local stream state.
    ///
    /// # Panics
    ///
    /// Panics only if a pending job loses its stream owner, which construction forbids.
    pub fn poll(&mut self) -> bool {
        if self.completed.is_some() {
            return true;
        }
        let result = match self.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return false,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(RuntimeErrorKind::Io("file worker stopped".to_owned()))
            }
        };
        let mut state = self
            .owner
            .stream_state()
            .expect("pending stream exists")
            .borrow_mut();
        let result = result.map(|mut done| {
            if let Some(file) = done.replacement.take() {
                let released = match &state.storage {
                    StreamStorage::Memory(bytes) => bytes.capacity(),
                    _ => 0,
                };
                state.storage = StreamStorage::File { file };
                self.owner.release_growth(released);
            }
            if let Some(cursor) = done.cursor {
                state.cursor = cursor;
            }
            done.result
        });
        state.busy = false;
        self.completed = Some(result);
        true
    }

    pub fn take_result(&mut self) -> Option<Result<StreamFileResult, RuntimeErrorKind>> {
        self.completed.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeErrorKind, StreamFileResult, Work, process};

    #[test]
    fn reading_past_eof_allocates_only_available_bytes() {
        let mut file = tempfile::tempfile().unwrap();
        std::io::Write::write_all(&mut file, b"x").unwrap();
        let result = process(
            Some(file),
            Work::Read {
                cursor: 0,
                length: usize::MAX,
            },
        )
        .unwrap();
        assert!(matches!(result.result, StreamFileResult::Bytes(bytes) if bytes == b"x"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn reads_virtual_regular_files_and_enforces_actual_byte_limits() {
        let path = "/proc/self/cmdline";
        assert_eq!(std::fs::metadata(path).unwrap().len(), 0);
        let result = process(
            Some(std::fs::File::open(path).unwrap()),
            Work::ReadAll {
                cursor: 0,
                limit: Some(0),
            },
        );
        assert!(
            matches!(result, Err(RuntimeErrorKind::Io(message)) if message == "stream read limit exceeded")
        );
        let result = process(
            Some(std::fs::File::open(path).unwrap()),
            Work::Read {
                cursor: 0,
                length: 1,
            },
        )
        .unwrap();
        assert!(matches!(result.result, StreamFileResult::Bytes(bytes) if bytes.len() == 1));
        let result = process(
            Some(std::fs::File::open(path).unwrap()),
            Work::Eof { cursor: 0 },
        )
        .unwrap();
        assert!(matches!(result.result, StreamFileResult::Bool(false)));
    }
}
