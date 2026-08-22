use std::fmt;
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A location in a UTF-8 source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    /// Zero-based byte span, when the parser reported one.
    pub span: Option<Range<usize>>,
    /// One-based line number.
    pub line: usize,
    /// One-based Unicode-scalar column.
    pub column: usize,
}

/// A structured source-configuration error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub location: Option<SourceLocation>,
    pub field: Option<String>,
    pub message: String,
}

impl Diagnostic {
    pub(crate) fn new(
        path: impl Into<PathBuf>,
        source: Option<&str>,
        span: Option<Range<usize>>,
        field: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        let location = source
            .zip(span)
            .map(|(source, span)| SourceLocation::from_offset(source, span.start, Some(span)));
        Self {
            path: path.into(),
            location,
            field,
            message: message.into(),
        }
    }

    pub(crate) fn at_field(
        path: &Path,
        source: &str,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let field = field.into();
        let needle = field.rsplit('.').next().unwrap_or(&field);
        let offset = find_field_offset(source, &field).unwrap_or(0);
        let span = offset..offset.saturating_add(needle.len());
        Self {
            path: path.to_path_buf(),
            location: Some(SourceLocation::from_offset(source, offset, Some(span))),
            field: Some(field),
            message: message.into(),
        }
    }
}

fn find_field_offset(source: &str, field: &str) -> Option<usize> {
    let parts = field.split('.').collect::<Vec<_>>();
    let key = parts.last()?;

    if parts.len() > 1 {
        let table = parts[..parts.len() - 1].join(".");
        let header = format!("[{table}]");
        if let Some(header_offset) = source.find(&header) {
            let body_start = header_offset + header.len();
            let body = &source[body_start..];
            let body_end = body
                .find("\n[")
                .map_or(source.len(), |offset| body_start + offset);
            for (relative, line) in
                source[body_start..body_end]
                    .split_inclusive('\n')
                    .scan(0, |offset, line| {
                        let current = *offset;
                        *offset += line.len();
                        Some((current, line))
                    })
            {
                let indentation = line.len() - line.trim_start().len();
                let trimmed = line.trim_start();
                if trimmed
                    .strip_prefix(key)
                    .is_some_and(|tail| tail.trim_start().starts_with('='))
                {
                    return Some(body_start + relative + indentation);
                }
            }
        }
    }

    let table = format!("[{field}]");
    source
        .find(&table)
        .map(|offset| offset + 1 + field.len() - key.len())
        .or_else(|| source.match_indices(key).next().map(|(offset, _)| offset))
}

impl SourceLocation {
    fn from_offset(source: &str, offset: usize, span: Option<Range<usize>>) -> Self {
        let offset = offset.min(source.len());
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix, |(_, tail)| tail)
            .chars()
            .count()
            + 1;
        Self { span, line, column }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path.display())?;
        if let Some(location) = &self.location {
            write!(formatter, ":{}:{}", location.line, location.column)?;
        }
        if let Some(field) = &self.field {
            write!(formatter, " ({field})")?;
        }
        write!(formatter, ": {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}
