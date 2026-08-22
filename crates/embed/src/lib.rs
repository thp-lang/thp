//! Safe host embedding boundary for THP requests.
#![allow(
    clippy::result_large_err,
    reason = "embedding failures intentionally return complete response and request statistics"
)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use thp_compiler::{compile_project_with_provider, compile_text};
use thp_metrics::{Metrics, Stage};
use thp_modules::ModuleSourceProvider;
use thp_runtime::{HeapStats, RequestInput};
use thp_vm::{ExecutionContext, Limits, VmError, execute_captured, execute_to};

#[derive(Clone, Copy, Debug, Default)]
pub struct Engine {
    limits: Limits,
}

#[derive(Clone, Debug)]
pub struct PreparedProject {
    project: thp_compiler::PreparedProject,
}

impl Engine {
    pub const fn new(limits: Limits) -> Self {
        Self { limits }
    }

    pub fn execute(&self, path: impl Into<PathBuf>, source: impl Into<String>) -> Response {
        let context = ExecutionContext {
            limits: self.limits,
            filesystem_base: PathBuf::new(),
            request_input: RequestInput::empty(),
        };
        self.execute_with_context(path, source, &context)
    }

    /// Compiles and verifies all modules returned by a source provider.
    ///
    /// # Errors
    ///
    /// Returns a compile- or host-error response when preparation fails.
    pub fn prepare_project(
        &self,
        request: &thp_compiler::ProjectRequest,
        provider: &dyn ModuleSourceProvider,
    ) -> Result<PreparedProject, Response> {
        let compilation =
            compile_project_with_provider(request, provider).map_err(|error| Response {
                status: Status::HostError,
                output: Vec::new(),
                error: error.to_string().into_bytes(),
                metrics: error.metrics,
                request: RequestStats::default(),
            })?;
        if !compilation.is_success() {
            return Err(Response {
                status: Status::CompileError,
                output: Vec::new(),
                error: compilation.rendered_diagnostics().into_bytes(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            });
        }
        let Some(project) = thp_compiler::PreparedProject::from_compilation(&compilation) else {
            return Err(Response {
                status: Status::HostError,
                output: Vec::new(),
                error: b"compiler reported success without a prepared project".to_vec(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            });
        };
        Ok(PreparedProject { project })
    }

    pub fn execute_prepared(&self, prepared: &PreparedProject) -> Response {
        let context = ExecutionContext {
            limits: self.limits,
            filesystem_base: PathBuf::new(),
            request_input: RequestInput::empty(),
        };
        self.execute_prepared_with_context(prepared, &context)
    }

    /// Executes a prepared project with request-local input and policy.
    pub fn execute_prepared_with_context(
        &self,
        prepared: &PreparedProject,
        context: &ExecutionContext,
    ) -> Response {
        let context = self.constrain_context(context);
        let mut metrics = Metrics::default();
        let execution = metrics.measure(Stage::PreparedExecution, || {
            execute_captured(&prepared.project.bytecode, &context)
        });
        match execution {
            Ok(execution) => Response {
                status: Status::Success,
                output: execution.output,
                error: Vec::new(),
                metrics,
                request: RequestStats {
                    heap: execution.heap,
                    input_bytes: context.request_input.consumed_bytes(),
                    output_bytes: execution.output_bytes,
                },
            },
            Err(failure) => {
                let error = prepared
                    .project
                    .sources
                    .get(prepared.project.entry_source)
                    .map_or_else(
                        || failure.error.to_string(),
                        |source| render_vm_error(source, &failure.error),
                    );
                Response {
                    status: Status::RuntimeError,
                    output: failure.output,
                    error: error.into_bytes(),
                    metrics,
                    request: RequestStats {
                        heap: failure.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: failure.output_bytes,
                    },
                }
            }
        }
    }

    /// Executes a prepared project while streaming output to `sink`.
    pub fn execute_prepared_to(
        &self,
        prepared: &PreparedProject,
        context: &ExecutionContext,
        sink: &mut dyn Write,
    ) -> StreamingResponse {
        let context = self.constrain_context(context);
        let mut metrics = Metrics::default();
        let execution = metrics.measure(Stage::PreparedExecution, || {
            execute_to(&prepared.project.bytecode, &context, sink)
        });
        match execution {
            Ok(execution) => StreamingResponse {
                status: Status::Success,
                error: Vec::new(),
                metrics,
                request: RequestStats {
                    heap: execution.heap,
                    input_bytes: context.request_input.consumed_bytes(),
                    output_bytes: execution.output_bytes,
                },
            },
            Err(failure) => {
                let error = prepared
                    .project
                    .sources
                    .get(prepared.project.entry_source)
                    .map_or_else(
                        || failure.error.to_string(),
                        |source| render_vm_error(source, &failure.error),
                    );
                StreamingResponse {
                    status: Status::RuntimeError,
                    error: error.into_bytes(),
                    metrics,
                    request: RequestStats {
                        heap: failure.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: failure.output_bytes,
                    },
                }
            }
        }
    }

    /// Compiles and executes a request with request-local VM context.
    pub fn execute_with_context(
        &self,
        path: impl Into<PathBuf>,
        source: impl Into<String>,
        context: &ExecutionContext,
    ) -> Response {
        let context = self.constrain_context(context);
        let mut compilation = compile_text(path, source);
        if !compilation.is_success() {
            return Response {
                status: Status::CompileError,
                output: Vec::new(),
                error: compilation.rendered_diagnostics().into_bytes(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            };
        }
        let Some(bytecode) = compilation.bytecode.as_ref() else {
            return Response {
                status: Status::HostError,
                output: Vec::new(),
                error: b"compiler reported success without bytecode".to_vec(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            };
        };
        let execution = compilation
            .metrics
            .measure(Stage::Vm, || execute_captured(bytecode, &context));
        match execution {
            Ok(execution) => {
                if let Some(measurement) = compilation.metrics.last_mut() {
                    measurement.set_output(
                        usize::try_from(execution.instructions).unwrap_or(usize::MAX),
                        execution.output.len(),
                    );
                }
                Response {
                    status: Status::Success,
                    output: execution.output,
                    error: Vec::new(),
                    metrics: compilation.metrics,
                    request: RequestStats {
                        heap: execution.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: execution.output_bytes,
                    },
                }
            }
            Err(failure) => {
                if let Some(measurement) = compilation.metrics.last_mut() {
                    measurement.set_output(
                        usize::try_from(failure.instructions).unwrap_or(usize::MAX),
                        failure.output.len(),
                    );
                }
                Response {
                    status: Status::RuntimeError,
                    output: failure.output,
                    error: render_vm_error(&compilation.source, &failure.error).into_bytes(),
                    metrics: compilation.metrics,
                    request: RequestStats {
                        heap: failure.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: failure.output_bytes,
                    },
                }
            }
        }
    }

    /// Compiles and executes a request while streaming output to `sink`.
    pub fn execute_to(
        &self,
        path: impl Into<PathBuf>,
        source: impl Into<String>,
        context: &ExecutionContext,
        sink: &mut dyn Write,
    ) -> StreamingResponse {
        let context = self.constrain_context(context);
        let mut compilation = compile_text(path, source);
        if !compilation.is_success() {
            return StreamingResponse {
                status: Status::CompileError,
                error: compilation.rendered_diagnostics().into_bytes(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            };
        }
        let Some(bytecode) = compilation.bytecode.as_ref() else {
            return StreamingResponse {
                status: Status::HostError,
                error: b"compiler reported success without bytecode".to_vec(),
                metrics: compilation.metrics,
                request: RequestStats::default(),
            };
        };
        let execution = compilation
            .metrics
            .measure(Stage::Vm, || execute_to(bytecode, &context, sink));
        match execution {
            Ok(execution) => {
                if let Some(measurement) = compilation.metrics.last_mut() {
                    measurement.set_output(
                        usize::try_from(execution.instructions).unwrap_or(usize::MAX),
                        usize::try_from(execution.output_bytes).unwrap_or(usize::MAX),
                    );
                }
                StreamingResponse {
                    status: Status::Success,
                    error: Vec::new(),
                    metrics: compilation.metrics,
                    request: RequestStats {
                        heap: execution.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: execution.output_bytes,
                    },
                }
            }
            Err(failure) => {
                if let Some(measurement) = compilation.metrics.last_mut() {
                    measurement.set_output(
                        usize::try_from(failure.instructions).unwrap_or(usize::MAX),
                        usize::try_from(failure.output_bytes).unwrap_or(usize::MAX),
                    );
                }
                StreamingResponse {
                    status: Status::RuntimeError,
                    error: render_vm_error(&compilation.source, &failure.error).into_bytes(),
                    metrics: compilation.metrics,
                    request: RequestStats {
                        heap: failure.heap,
                        input_bytes: context.request_input.consumed_bytes(),
                        output_bytes: failure.output_bytes,
                    },
                }
            }
        }
    }

    /// Processes one request through a host-provided SAPI adapter.
    ///
    /// # Errors
    ///
    /// Returns adapter failures while reading the request or writing response.
    pub fn serve(&self, sapi: &mut impl Sapi) -> Result<(), SapiError> {
        let request = sapi.read_request()?;
        let context = ExecutionContext {
            limits: self.limits,
            filesystem_base: PathBuf::new(),
            request_input: request.body,
        };
        let mut writer = SapiWriter { sapi };
        let response = self.execute_to(&request.path, request.source, &context, &mut writer);
        writer.sapi.finish_response(response)
    }

    fn constrain_context(&self, context: &ExecutionContext) -> ExecutionContext {
        ExecutionContext {
            limits: Limits {
                max_instructions: stricter_limit(
                    self.limits.max_instructions,
                    context.limits.max_instructions,
                ),
                max_execution: stricter_limit(
                    self.limits.max_execution,
                    context.limits.max_execution,
                ),
                max_heap_bytes: stricter_limit(
                    self.limits.max_heap_bytes,
                    context.limits.max_heap_bytes,
                ),
                max_input_bytes: stricter_limit(
                    self.limits.max_input_bytes,
                    context.limits.max_input_bytes,
                ),
                max_input_time: stricter_limit(
                    self.limits.max_input_time,
                    context.limits.max_input_time,
                ),
                max_stack_depth: stricter_limit(
                    self.limits.max_stack_depth,
                    context.limits.max_stack_depth,
                ),
                max_open_handles: stricter_limit(
                    self.limits.max_open_handles,
                    context.limits.max_open_handles,
                ),
            },
            filesystem_base: context.filesystem_base.clone(),
            request_input: context.request_input.clone(),
        }
    }
}

fn stricter_limit<T: Ord + Copy>(left: Option<T>, right: Option<T>) -> Option<T> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(limit), None) | (None, Some(limit)) => Some(limit),
        (None, None) => None,
    }
}

fn render_vm_error(source: &thp_diagnostics::SourceFile, error: &VmError) -> String {
    match error {
        VmError::Runtime(runtime) => {
            let (line, column) = source.line_column(runtime.span.start as usize);
            format!("{}:{line}:{column}: {runtime}", source.path().display())
        }
        other => other.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Status {
    Success = 0,
    CompileError = 1,
    RuntimeError = 2,
    HostError = 3,
}

#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub output: Vec<u8>,
    pub error: Vec<u8>,
    pub metrics: Metrics,
    pub request: RequestStats,
}

#[derive(Debug)]
pub struct StreamingResponse {
    pub status: Status,
    pub error: Vec<u8>,
    pub metrics: Metrics,
    pub request: RequestStats,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RequestStats {
    pub heap: HeapStats,
    pub input_bytes: u64,
    pub output_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct Request {
    pub path: PathBuf,
    pub source: String,
    pub body: RequestInput,
}

pub trait Sapi {
    /// Reads one complete request.
    ///
    /// # Errors
    ///
    /// Returns transport, decoding, or host-policy failures.
    fn read_request(&mut self) -> Result<Request, SapiError>;

    /// Writes one output chunk, synchronously applying host backpressure.
    ///
    /// # Errors
    ///
    /// Returns transport or host failures.
    fn write_output(&mut self, bytes: &[u8]) -> Result<(), SapiError>;

    /// Completes a response after execution has stopped.
    ///
    /// # Errors
    ///
    /// Returns transport or host failures while finalizing the response.
    fn finish_response(&mut self, response: StreamingResponse) -> Result<(), SapiError>;
}

struct SapiWriter<'a, S: Sapi> {
    sapi: &'a mut S,
}

impl<S: Sapi> Write for SapiWriter<'_, S> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.sapi
            .write_output(bytes)
            .map_err(|error| io::Error::other(error.to_string()))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SapiError {
    pub message: String,
}

impl SapiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SapiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SapiError {}

pub fn path_label(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::path::PathBuf;

    use thp_runtime::RequestInput;
    use thp_vm::{ExecutionContext, Limits};

    use super::{Engine, Status};

    #[test]
    fn embedding_api_returns_binary_output_without_printing() {
        let response = Engine::default().execute("embed.thp", "<?thp\necho \"a\\x00b\";");
        assert_eq!(response.status, Status::Success);
        assert_eq!(response.output, b"a\0b");
        assert!(response.error.is_empty());
    }

    #[test]
    fn embedding_api_returns_compile_diagnostics() {
        let response = Engine::default().execute("embed.thp", "<?thp\n$value = ;");
        assert_eq!(response.status, Status::CompileError);
        assert!(!response.error.is_empty());
    }

    #[test]
    fn context_execution_retains_output_before_runtime_errors() {
        let response = Engine::default().execute_with_context(
            "embed.thp",
            "<?thp\necho \"before\\n\";\n$value = 9223372036854775807 + 1;",
            &ExecutionContext {
                limits: Limits::default(),
                filesystem_base: PathBuf::new(),
                request_input: RequestInput::empty(),
            },
        );
        assert_eq!(response.status, Status::RuntimeError);
        assert_eq!(response.output, b"before\n");
        assert!(!response.error.is_empty());
    }

    #[test]
    fn engine_policy_cannot_be_relaxed_by_a_request_context() {
        let engine = Engine::new(Limits {
            max_input_bytes: Some(2),
            ..Limits::default()
        });
        let response = engine.execute_with_context(
            "limits.thp",
            "<?thp\necho \"unreached\";",
            &ExecutionContext {
                request_input: RequestInput::from_bytes(b"body".to_vec(), None, None)
                    .expect("unlimited body"),
                ..ExecutionContext::default()
            },
        );
        assert_eq!(response.status, Status::RuntimeError);
        assert_eq!(response.request.input_bytes, 0);
        assert!(String::from_utf8_lossy(&response.error).contains("2 byte limit"));
    }

    #[test]
    fn repeated_requests_release_heap_cells_and_handles_on_success_and_failure() {
        let engine = Engine::default();
        for source in [
            "<?thp\n$stream = MemoryStream::open(\"payload\"); echo \"ok\";",
            "<?thp\n$stream = MemoryStream::open(\"payload\"); $value = 9223372036854775807 + 1;",
        ] {
            let response = engine.execute("teardown.thp", source);
            assert_eq!(response.request.heap.live_cells, 0);
            assert_eq!(response.request.heap.open_handles, 0);
            assert!(response.request.heap.peak_open_handles >= 1);
        }
    }

    #[derive(Default)]
    struct CountingSink {
        bytes: u64,
    }

    impl Write for CountingSink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes += bytes.len() as u64;
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn streaming_embedding_does_not_return_or_heap_account_output() {
        let mut sink = CountingSink::default();
        let context = ExecutionContext::default();
        let response = Engine::default().execute_to(
            "stream.thp",
            "<?thp\n$index = 0; while ($index < 1000) { echo \"chunk\"; $index = $index + 1; }",
            &context,
            &mut sink,
        );
        assert_eq!(response.status, Status::Success);
        assert_eq!(response.request.output_bytes, 5000);
        assert_eq!(sink.bytes, 5000);
        assert!(response.request.heap.peak_bytes < 128 * 1024);
    }
}
