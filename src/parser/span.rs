use crate::parser::line::{CodeKind, Line};

#[derive(Debug, Clone)]
pub enum Span<'a> {
    Text {
        start: usize,
        end: usize,
        quoting_layer: usize,
        lines: &'a [Line<'a>],
    },
    DiffHeader {
        start: usize,
        end: usize,
        quoting_layer: usize,
        lines: &'a [Line<'a>],
    },
    DiffMetadata {
        start: usize,
        end: usize,
        quoting_layer: usize,
        lines: &'a [Line<'a>],
    },
    HunkHeader {
        start: usize,
        end: usize,
        quoting_layer: usize,
        lines: &'a [Line<'a>],
    },
    Code {
        start: usize,
        end: usize,
        quoting_layer: usize,
        kind: CodeKind,
        lines: &'a [Line<'a>],
    },
}

/// Create a vector of `Span`-s from a vector of `Line`-s
pub fn build_spans<'a>(lines: &'a [Line<'a>]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    if lines.is_empty() {
        return spans;
    }

    let mut start_idx = 0;

    for i in 1..=lines.len() {
        if i == lines.len()
            || !&lines[start_idx].same_quoting_layer(&lines[i])
            || !lines[start_idx].same_variant(&lines[i])
        {
            let span = match &lines[start_idx] {
                Line::Text {
                    offset,
                    quoting_layer,
                    ..
                } => Span::Text {
                    start: *offset,
                    end: lines[i - 1].get_end_offset(),
                    quoting_layer: *quoting_layer,
                    lines: &lines[start_idx..i],
                },
                Line::DiffHeader {
                    offset,
                    quoting_layer,
                    ..
                } => Span::DiffHeader {
                    start: *offset,
                    end: lines[i - 1].get_end_offset(),
                    quoting_layer: *quoting_layer,
                    lines: &lines[start_idx..i],
                },
                Line::DiffMetadata {
                    offset,
                    quoting_layer,
                    ..
                } => Span::DiffMetadata {
                    start: *offset,
                    end: lines[i - 1].get_end_offset(),
                    quoting_layer: *quoting_layer,
                    lines: &lines[start_idx..i],
                },
                Line::HunkHeader {
                    offset,
                    quoting_layer,
                    ..
                } => Span::HunkHeader {
                    start: *offset,
                    end: lines[i - 1].get_end_offset(),
                    quoting_layer: *quoting_layer,
                    lines: &lines[start_idx..i],
                },
                Line::Code {
                    offset,
                    quoting_layer,
                    kind,
                    ..
                } => Span::Code {
                    start: *offset,
                    end: lines[i - 1].get_end_offset(),
                    quoting_layer: *quoting_layer,
                    kind: kind.clone(),
                    lines: &lines[start_idx..i],
                },
            };

            spans.push(span);
            start_idx = i;
        }
    }

    spans
}
