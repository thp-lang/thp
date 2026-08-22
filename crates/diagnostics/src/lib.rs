//! Structured diagnostics shared by every THP compiler phase.

use std::fmt;
use std::fmt::Write as _;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Stable identity for a source within one compilation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId(pub u32);

/// Immutable sources addressed by compact IDs for cross-file diagnostics.
#[derive(Clone, Debug, Default)]
pub struct SourceMap {
    sources: Vec<SourceFile>,
}

impl SourceMap {
    /// Adds one source and returns its compact identity.
    ///
    /// # Panics
    ///
    /// Panics if one compilation contains more than `u32::MAX` source files.
    pub fn add(&mut self, source: SourceFile) -> SourceId {
        let id = SourceId(
            u32::try_from(self.sources.len())
                .expect("a compilation is limited to u32::MAX sources"),
        );
        self.sources.push(source);
        id
    }

    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.sources.get(id.0 as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceId, &SourceFile)> {
        self.sources
            .iter()
            .enumerate()
            .map(|(index, source)| (SourceId(u32::try_from(index).unwrap_or(u32::MAX)), source))
    }
}

/// A half-open byte range in a source file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    /// Creates a span from byte offsets.
    ///
    /// # Panics
    ///
    /// Panics when `end < start` or an offset does not fit in 32 bits.
    pub fn new(start: usize, end: usize) -> Self {
        assert!(end >= start, "a source span cannot end before it starts");
        Self {
            start: u32::try_from(start).expect("THP source files are limited to 4 GiB"),
            end: u32::try_from(end).expect("THP source files are limited to 4 GiB"),
        }
    }

    pub fn empty(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    pub fn range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    #[must_use]
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// An immutable UTF-8 source file.
#[derive(Clone, Debug)]
pub struct SourceFile {
    path: Arc<PathBuf>,
    text: Arc<str>,
    line_starts: Arc<[u32]>,
}

impl SourceFile {
    /// Creates a source file and its line index.
    ///
    /// # Panics
    ///
    /// Panics when the source is 4 GiB or larger.
    pub fn new(path: impl Into<PathBuf>, text: impl Into<Arc<str>>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        for (offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(
                    u32::try_from(offset + 1).expect("THP source files are limited to 4 GiB"),
                );
            }
        }
        Self {
            path: Arc::new(path.into()),
            text,
            line_starts: line_starts.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns a one-based line and Unicode-scalar column.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.text.len());
        let line_index = self
            .line_starts
            .partition_point(|start| *start as usize <= offset)
            .saturating_sub(1);
        let line_start = self.line_starts[line_index] as usize;
        let column = self.text[line_start..offset].chars().count() + 1;
        (line_index + 1, column)
    }

    pub fn line_text(&self, one_based_line: usize) -> Option<&str> {
        let index = one_based_line.checked_sub(1)?;
        let start = *self.line_starts.get(index)? as usize;
        let end = self
            .line_starts
            .get(index + 1)
            .map_or(self.text.len(), |offset| *offset as usize);
        Some(self.text[start..end].trim_end_matches(['\n', '\r']))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    pub source: Option<SourceId>,
    pub span: Span,
    pub message: Option<String>,
}

/// A compiler diagnostic independent of its rendering destination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub phase: &'static str,
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(
        phase: &'static str,
        code: &'static str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            phase,
            code,
            severity: Severity::Error,
            message: message.into(),
            labels: vec![Label {
                source: None,
                span,
                message: None,
            }],
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label {
            source: None,
            span,
            message: Some(message.into()),
        });
        self
    }

    #[must_use]
    pub fn with_source_label(
        mut self,
        source: SourceId,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        self.labels.push(Label {
            source: Some(source),
            span,
            message: Some(message.into()),
        });
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Renders a compact diagnostic with a source line and caret.
    pub fn render(&self, source: &SourceFile) -> String {
        self.render_with(|_| source)
    }

    /// Renders every label against its owning file. Labels without an explicit
    /// source use `default_source`.
    pub fn render_with_sources(&self, sources: &SourceMap, default_source: SourceId) -> String {
        let Some(fallback) = sources.get(default_source) else {
            return format!(
                "<unknown>:1:1: {}[{}]: {}\n",
                self.severity, self.code, self.message
            );
        };
        self.render_with(|id| id.and_then(|id| sources.get(id)).unwrap_or(fallback))
    }

    fn render_with<'a>(&self, source_for: impl Fn(Option<SourceId>) -> &'a SourceFile) -> String {
        let mut rendered = String::new();

        let primary_label = self.labels.first();
        let source = source_for(primary_label.and_then(|label| label.source));
        let primary = primary_label.map(|label| label.span);
        let (line, column) = primary.map_or((1, 1), |span| source.line_column(span.start as usize));
        let _ = writeln!(
            rendered,
            "{}:{}:{}: {}[{}]: {}",
            source.path().display(),
            line,
            column,
            self.severity,
            self.code,
            self.message
        );
        if let Some(text) = source.line_text(line) {
            let _ = writeln!(rendered, " {line:>4} | {text}");
            let width = primary.map_or(1, |span| {
                usize::max(1, (span.end.saturating_sub(span.start)) as usize)
            });
            let message = self
                .labels
                .first()
                .and_then(|label| label.message.as_deref())
                .map_or_else(String::new, |message| format!(" {message}"));
            let _ = writeln!(
                rendered,
                "      | {}{}{}",
                " ".repeat(column.saturating_sub(1)),
                "^".repeat(width.min(text.len().saturating_sub(column - 1).max(1))),
                message,
            );
        }
        for label in self.labels.iter().skip(1) {
            let label_source = source_for(label.source);
            let (label_line, label_column) = label_source.line_column(label.span.start as usize);
            let _ = writeln!(
                rendered,
                " {}:{}:{}: related location",
                label_source.path().display(),
                label_line,
                label_column
            );
            if let Some(text) = label_source.line_text(label_line) {
                let width = usize::max(1, label.span.end.saturating_sub(label.span.start) as usize);
                let message = label
                    .message
                    .as_deref()
                    .map_or_else(String::new, |message| format!(" {message}"));
                let _ = writeln!(rendered, " {label_line:>4} | {text}");
                let _ = writeln!(
                    rendered,
                    "      | {}{}{}",
                    " ".repeat(label_column.saturating_sub(1)),
                    "^".repeat(width.min(text.len().saturating_sub(label_column - 1).max(1))),
                    message,
                );
            }
        }
        for note in &self.notes {
            let _ = writeln!(rendered, " note: {note}");
        }
        rendered
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, SourceFile, Span};

    #[test]
    fn locations_count_unicode_columns() {
        let source = SourceFile::new("test.thp", "<?thp\n$π = 1;\n");
        assert_eq!(source.line_column(9), (2, 3));
    }

    #[test]
    fn renders_source_location_and_code() {
        let source = SourceFile::new("test.thp", "<?thp\n$x = nope;\n");
        let diagnostic = Diagnostic::error("typing", "T1001", Span::new(11, 15), "unknown name");
        let rendered = diagnostic.render(&source);
        assert!(rendered.contains("test.thp:2:6: error[T1001]"));
        assert!(rendered.contains("^^^^"));
    }

    #[test]
    fn renders_related_labels_and_notes() {
        let source = SourceFile::new("test.thp", "<?thp\nfirst\nsecond\n");
        let diagnostic = Diagnostic::error("typing", "T1002", Span::new(12, 18), "duplicate")
            .with_label(Span::new(6, 11), "first occurrence is here")
            .with_note("choose one occurrence");
        let rendered = diagnostic.render(&source);
        assert!(rendered.contains("test.thp:2:1: related location"));
        assert!(rendered.contains("first occurrence is here"));
        assert!(rendered.contains("note: choose one occurrence"));
    }
}
