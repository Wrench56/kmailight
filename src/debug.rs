#![allow(dead_code)]

#[cfg(debug_assertions)]
use crate::parser::line::Line;

#[cfg(debug_assertions)]
/// Dump the tree for debugging
pub fn dump_tree(node: tree_sitter::Node, source: &str) {
    fn _dump_tree(node: tree_sitter::Node, source: &str, indent: usize) {
        let indent_str = "  ".repeat(indent);
        let kind = node.kind();
        let span = &source[node.start_byte()..node.end_byte()];
        println!("{indent_str}{kind}: {:?}", span.trim());

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            _dump_tree(child, source, indent + 1);
        }
    }
    _dump_tree(node, source, 0);
}

#[cfg(debug_assertions)]
/// Pretty-print the valid and invalid chunks returned by `collect_non_error_chunks`.
pub fn print_chunks(chunks: &[(usize, usize)], src: &str) {
    const RED: &str = "\x1b[1;31m";
    const RESET: &str = "\x1b[0m";

    println!("\n{:=^80}", " Chunks ");

    /* True means the chunk is invalid (ERROR) */
    let mut all_chunks = Vec::new();
    let mut last_end = 0;

    for &(start, end) in chunks {
        if start > last_end {
            all_chunks.push((last_end, start, true));
        }
        all_chunks.push((start, end, false));
        last_end = end;
    }

    if last_end < src.len() {
        all_chunks.push((last_end, src.len(), true));
    }

    for (i, &(start, end, is_error)) in all_chunks.iter().enumerate() {
        let len = end - start;
        let raw_snippet = &src[start..end];
        let mut lines = raw_snippet.lines();

        let label = if is_error { "ERROR" } else { "" };
        let prefix = format!(
            " {idx:>3}. [{start:>5},{end:<5}] len={len:<5} {RED}{label:<6}{RESET}  ",
            idx = i + 1,
            start = start,
            end = end - 1,
            len = len,
            label = label
        );

        if let Some(first_line) = lines.next() {
            print!("{prefix}");
            println!("{first_line}");

            let indent_width = prefix.chars().count() - RED.len() - RESET.len();
            let code_offset = first_line.chars().take_while(|c| c.is_whitespace()).count();
            let indent = " ".repeat(indent_width + code_offset);

            for line in lines {
                println!("{indent}{line}");
            }
        }
    }

    println!("{:=^80}\n", "");
}

#[cfg(debug_assertions)]
/// Pretty-print for lines
pub fn print_lines(lines: &[Line]) {
    fn preview(s: &str) -> String {
        let mut p = s.to_string();
        if p.len() > 80 {
            p.truncate(77);
            p.push_str("...");
        }
        format!("{:?}", p)
    }

    fn shorten(s: &str, max: usize) -> String {
        if s.len() <= max {
            format!("{:width$}", s, width = max)
        } else {
            format!("...{}", &s[s.len() - max + 3..])
        }
    }

    fn format_prefix(line: &Line) -> String {
        match line {
            Line::Text {
                offset,
                quoting_layer,
                length,
                ..
            } => {
                format!(
                    "TXT  off:{:>5}  q:{:<2}  len:{:>4}",
                    offset, quoting_layer, length
                )
            }
            Line::DiffHeader {
                offset,
                quoting_layer,
                file_path,
                length,
                ..
            } => {
                format!(
                    "DIFF off:{:>5}  q:{:<2}  len:{:>4}              file:{:<20}",
                    offset,
                    quoting_layer,
                    length,
                    shorten(file_path, 20)
                )
            }
            Line::DiffMetadata {
                offset,
                quoting_layer,
                length,
                ..
            } => {
                format!(
                    "META off:{:>5}  q:{:<2}  len:{:>4}",
                    offset, quoting_layer, length
                )
            }
            Line::HunkHeader {
                offset,
                quoting_layer,
                file_path,
                language,
                length,
                ..
            } => {
                format!(
                    "HUNK off:{:>5}  q:{:<2}  len:{:>4}              file:{:<20} lang:{:<7}",
                    offset,
                    quoting_layer,
                    length,
                    shorten(file_path, 20),
                    language,
                )
            }
            Line::Code {
                offset,
                quoting_layer,
                kind,
                file_path,
                language,
                length,
                ..
            } => {
                format!(
                    "CODE off:{:>5}  q:{:<2}  len:{:>4}  kind:{:<7} file:{:<20} lang:{:<7}",
                    offset,
                    quoting_layer,
                    length,
                    format!("{:?}", kind),
                    shorten(file_path, 20),
                    language
                )
            }
        }
    }

    let rows: Vec<(String, String)> = lines
        .iter()
        .map(|line| {
            let prefix = format_prefix(line);
            let raw = match line {
                Line::Text { raw, .. }
                | Line::DiffHeader { raw, .. }
                | Line::DiffMetadata { raw, .. }
                | Line::HunkHeader { raw, .. }
                | Line::Code { raw, .. } => preview(raw),
            };
            (prefix, raw)
        })
        .collect();

    let max_prefix_len = rows.iter().map(|(p, _)| p.len()).max().unwrap_or(0);

    for (prefix, raw) in rows {
        println!("{:<width$}  raw: {}", prefix, raw, width = max_prefix_len);
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
/// No-op for release builds
pub fn dump_tree(_: tree_sitter::Node, _: &str) {}

#[cfg(not(debug_assertions))]
#[inline(always)]
/// No-op for release builds
pub fn print_chunks(_: &[(usize, usize)], _: &str) {}

#[cfg(not(debug_assertions))]
#[inline(always)]
/// No-op for release builds
pub fn print_lines(_: &Vec<Line>) {}
