use std::env;
use std::fs;
use std::io::{self, Read};

use tree_sitter::Parser;
use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};
use tree_sitter_c;

/// Main entry point
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_code = if let Some(path) = env::args().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };
    let source_bytes = source_code.as_bytes();

    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into())?;
    let tree = parser.parse(&source_code, None).unwrap();
    let root = tree.root_node();

    let mut config = HighlightConfiguration::new(
        tree_sitter_c::LANGUAGE.into(),
        "c",
        tree_sitter_c::HIGHLIGHT_QUERY,
        "",
        tree_sitter_c::TAGS_QUERY,
    )?;
    config.configure(&[
        "function", "type", "string", "keyword", "number",
        "comment", "constant", "operator", "variable",
    ]);

    let mut highlighter = Highlighter::new();

    let valid_chunks = collect_non_error_chunks(root);
    let mut last_end = 0;

    for (start, end) in valid_chunks {
        print!("{}", &source_code[last_end..start]);

        let slice = &source_bytes[start..end];
        let base = start;

        let events = highlighter.highlight(&config, slice, None, |_| None)?;

        for event in events {
            match event? {
                HighlightEvent::Source { start, end } => {
                    print!("{}", std::str::from_utf8(&source_bytes[base + start..base + end])?);
                }
                HighlightEvent::HighlightStart(s) => {
                    print!("{}", ansi_for_class(s.0));
                }
                HighlightEvent::HighlightEnd => {
                    print!("\x1b[0m");
                }
            }
        }

        last_end = end;
    }

    print!("{}", &source_code[last_end..]);
    println!();

    Ok(())
}

/// Collect non-error chunks
fn collect_non_error_chunks(root: tree_sitter::Node) -> Vec<(usize, usize)> {
    let mut chunks = Vec::new();
    let mut cursor = root.walk();
    let mut current_start = None;

    for child in root.children(&mut cursor) {
        if child.kind() == "ERROR" {
            if let Some(start) = current_start.take() {
                chunks.push((start, child.start_byte()));
            }
        } else {
            if current_start.is_none() {
                current_start = Some(child.start_byte());
            }
        }
    }

    if let Some(start) = current_start {
        chunks.push((start, root.end_byte()));
    }

    chunks
}

/// Convert highlight class ID to ANSI color
fn ansi_for_class(class: usize) -> &'static str {
    match class {
        0 => "\x1b[1;34m", // function
        1 => "\x1b[1;36m", // type
        2 => "\x1b[0;32m", // string
        3 => "\x1b[1;35m", // keyword
        4 => "\x1b[0;36m", // number
        5 => "\x1b[0;90m", // comment
        6 => "\x1b[1;33m", // constant
        7 => "\x1b[1;31m", // operator
        8 => "\x1b[0m",    // default
        _ => "\x1b[0m",
    }
}
