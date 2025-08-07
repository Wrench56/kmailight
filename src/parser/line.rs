#[derive(Debug, Clone)]
pub enum CodeKind {
    Add,
    Remove,
    Context,
}

/// An enum representing the state of the parser
///
/// `Text` progresses into `Diff` when a diff header is found,
/// `Diff` progresses into `Hunk` when a hunk header is found,
/// and `Hunk` progresses into `Code` right after the hunk header line.
/// `Diff` state also includes the diff metadata lines.
/// When in `Code` state, it checks for hunk headers again, which will reset the state to `Hunk`.
/// Currently, there are no other ways to break out of the `Code` state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    Text,
    Diff,
    Hunk,
    Code,
}

/// Maintains parsing state for a specific quoting layer
///
/// The parser tracks multiple quoting layers in parallel because
/// context lines, inline comments, nested quoted diffs, etc. can appear
/// interleaved in email patches. Each quoting layer has its own
/// independent state machine so that each layer state is independent
/// of the others.
struct LayerState {
    state: State,
    file_path: String,
    language: String,
}

#[derive(Debug, Clone)]
pub enum Line<'a> {
    Text {
        offset: usize,
        length: usize,
        quoting_layer: usize,
        raw: &'a str,
    },
    DiffHeader {
        offset: usize,
        length: usize,
        quoting_layer: usize,
        file_path: String,
        raw: &'a str,
    },
    DiffMetadata {
        offset: usize,
        length: usize,
        quoting_layer: usize,
        raw: &'a str,
    },
    HunkHeader {
        offset: usize,
        length: usize,
        quoting_layer: usize,
        file_path: String,
        language: String,
        raw: &'a str,
    },
    Code {
        offset: usize,
        length: usize,
        quoting_layer: usize,
        kind: CodeKind,
        file_path: String,
        language: String,
        raw: &'a str,
    },
}

impl Line<'_> {
    /// Parse all lines from the given source code
    ///
    /// Each line has the fields `offset`, `length`, `quoting_layer`, and `raw`.
    /// On top of that, `DiffHeader`, `HunkHeader`, and `Code` lines
    /// have the additional field `file_path`.
    /// `HunkHeader` and `Code` lines also have the field `language`.
    /// The `kind` field in `Code` lines indicates whether the line is an addition (`+`), a removal (`-`), or context (no sign)
    /// based on the diff format.
    pub fn parse_lines(source: &str) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut offset = 0usize;

        let mut layers: Vec<Option<LayerState>> = Vec::new();

        for raw in source.lines() {
            let len = raw.len() + 1;
            let ql = quoting_layer(raw);
            let line = raw.trim_start_matches('>');
            let trimmed = line.trim_start();

            /* Dynamically resize layers vector for infinite quoting layers */
            if ql >= layers.len() {
                layers.resize_with(ql + 1, || None);
            }

            let entry = layers[ql].get_or_insert_with(|| LayerState {
                state: State::Text,
                file_path: String::new(),
                language: "Unknown".to_string(),
            });

            match entry.state {
                State::Text => {
                    if trimmed.starts_with("diff --git") {
                        entry.state = State::Diff;
                        entry.file_path = extract_file_path(trimmed);
                        entry.language = detect_language(&entry.file_path);
                        lines.push(Line::DiffHeader {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            file_path: entry.file_path.clone(),
                            raw,
                        });
                    } else {
                        lines.push(Line::Text {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            raw,
                        });
                    }
                }
                State::Diff => {
                    if trimmed.starts_with("@@") {
                        entry.state = State::Hunk;
                        lines.push(Line::HunkHeader {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            file_path: entry.file_path.clone(),
                            language: entry.language.clone(),
                            raw,
                        });
                    } else {
                        lines.push(Line::DiffMetadata {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            raw,
                        });
                    }
                }
                State::Hunk | State::Code => {
                    if trimmed.starts_with("@@") {
                        entry.state = State::Hunk;
                        lines.push(Line::HunkHeader {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            file_path: entry.file_path.clone(),
                            language: entry.language.clone(),
                            raw,
                        });
                    } else {
                        entry.state = State::Code;
                        lines.push(Line::Code {
                            offset,
                            length: len,
                            quoting_layer: ql,
                            kind: match_code_kind(trimmed).unwrap(),
                            file_path: entry.file_path.clone(),
                            language: entry.language.clone(),
                            raw,
                        });
                    }
                }
            }

            offset += len;
        }

        lines
    }

    /// Get the raw line text
    pub fn get_raw(&self) -> &str {
        match self {
            Line::Text { raw, .. } => raw,
            Line::DiffHeader { raw, .. } => raw,
            Line::DiffMetadata { raw, .. } => raw,
            Line::HunkHeader { raw, .. } => raw,
            Line::Code { raw, .. } => raw,
        }
    }

    /// Get the quoting layer of the line
    pub fn get_quoting_layer(&self) -> usize {
        match self {
            Line::Text { quoting_layer, .. }
            | Line::DiffHeader { quoting_layer, .. }
            | Line::DiffMetadata { quoting_layer, .. }
            | Line::HunkHeader { quoting_layer, .. }
            | Line::Code { quoting_layer, .. } => *quoting_layer,
        }
    }

    /// Check if two lines belong to the same quoting layer
    #[inline]
    pub fn same_quoting_layer(&self, other: &Self) -> bool {
        self.get_quoting_layer() == other.get_quoting_layer()
    }

    /// Check if two lines belong to the same kind (e.g.: Text, DiffHeader, Code, etc.)
    pub fn same_variant(&self, other: &Self) -> bool {
        use Line::*;
        matches!(
            (self, other),
            (Text { .. }, Text { .. })
                | (DiffHeader { .. }, DiffHeader { .. })
                | (DiffMetadata { .. }, DiffMetadata { .. })
                | (HunkHeader { .. }, HunkHeader { .. })
                | (Code { .. }, Code { .. })
        )
    }
}

/// Get the current quoting layer of a line
///
/// Every `>` followed by a whitespace or another `>` counts as a layer.
/// In case the `>` is followed by a non-whitespace character, it is not counted.
/// This is to counter edge cases where symbols as `->` are splitted across lines.
///
/// I doubt that this is a perfect solution, but it works for now.
#[inline]
fn quoting_layer(line: &str) -> usize {
    let mut count = 0;
    let mut seen_non_ws = false;

    for c in line.chars() {
        match c {
            '>' => {
                if seen_non_ws {
                    break;
                }
                count += 1;
            }
            ' ' | '\t' => continue,
            _ => break,
        }
        seen_non_ws = true;
    }

    count
}

/// Extract the file path from a diff line
///
/// This is quite volatile,but it works for common diff cases.
#[inline]
fn extract_file_path(diff_line: &str) -> String {
    diff_line
        .split_whitespace()
        .nth(2)
        .unwrap_or("unknown")
        .trim_start_matches("a/")
        .to_string()
}

/// Detect the language based on the file extension
#[inline]
fn detect_language(file_path: &str) -> String {
    match file_path.rsplit('.').next() {
        Some("c") => "C",
        Some("h") => "C Header",
        Some("rs") => "Rust",
        Some("py") => "Python",
        _ => "Unknown",
    }
    .to_string()
}

/// Get diff-ed code kind: Add (`+`), Remove (`-`), or Context (no sign)
#[inline]
fn match_code_kind(line: &str) -> Option<CodeKind> {
    let first = line.chars().find(|&c| c != ' ' && c != '\t');
    match first {
        Some('+') => Some(CodeKind::Add),
        Some('-') => Some(CodeKind::Remove),
        Some(_) => Some(CodeKind::Context),
        None => Some(CodeKind::Context),
    }
}
