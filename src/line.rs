#[derive(Debug, Clone)]
pub enum CodeKind {
    Add,
    Remove,
    Context,
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

        let mut lines = Vec::new();
        let mut offset = 0usize;
        let mut state = State::Text;

        let mut current_file = String::new();
        let mut current_lang = String::from("Unknown");

        for raw in source.lines() {
            let len = raw.len() + 1;
            let quoting_layer = quoting_layer(raw);
            let line = raw.trim_start_matches('>');
            let trimmed = line.trim_start();

            match state {
                State::Text => {
                    if trimmed.starts_with("diff --git") {
                        state = State::Diff;
                        current_file = extract_file_path(trimmed);
                        current_lang = detect_language(&current_file);

                        lines.push(Line::DiffHeader {
                            offset,
                            length: len,
                            quoting_layer,
                            file_path: current_file.clone(),
                            raw,
                        });
                    } else {
                        lines.push(Line::Text {
                            offset,
                            length: len,
                            quoting_layer,
                            raw,
                        });
                    }
                }

                State::Diff => {
                    if trimmed.starts_with("@@") {
                        state = State::Hunk;
                        lines.push(Line::HunkHeader {
                            offset,
                            length: len,
                            quoting_layer,
                            file_path: current_file.clone(),
                            language: current_lang.clone(),
                            raw,
                        });
                    } else {
                        lines.push(Line::DiffMetadata {
                            offset,
                            length: len,
                            quoting_layer,
                            raw,
                        });
                    }
                }

                State::Hunk | State::Code => {
                    if trimmed.starts_with("@@") {
                        state = State::Hunk;
                        lines.push(Line::HunkHeader {
                            offset,
                            length: len,
                            quoting_layer,
                            file_path: current_file.clone(),
                            language: current_lang.clone(),
                            raw,
                        });
                    } else {
                        state = State::Code;
                        lines.push(Line::Code {
                            offset,
                            length: len,
                            quoting_layer,
                            kind: match_code_kind(trimmed).unwrap(),
                            file_path: current_file.clone(),
                            language: current_lang.clone(),
                            raw,
                        });
                    }
                }
            }

            offset += len;
        }

        lines
    }

    pub fn get_raw(&self) -> &str {
        match self {
            Line::Text { raw, .. } => raw,
            Line::DiffHeader { raw, .. } => raw,
            Line::DiffMetadata { raw, .. } => raw,
            Line::HunkHeader { raw, .. } => raw,
            Line::Code { raw, .. } => raw,
        }
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
