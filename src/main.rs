use std::env;
use std::fs;
use std::io::{self, Read};

use tree_sitter_highlight::HighlightConfiguration;

mod debug;
mod line;

use crate::line::Line;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_code = if let Some(path) = env::args().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let mut config = HighlightConfiguration::new(
        tree_sitter_c::LANGUAGE.into(),
        "c",
        tree_sitter_c::HIGHLIGHT_QUERY,
        "",
        tree_sitter_c::TAGS_QUERY,
    )?;

    config.configure(&[
        "function", "type", "string", "keyword", "number", "comment", "constant", "operator",
        "variable",
    ]);

    let lines = Line::parse_lines(&source_code);
    debug::print_lines(&lines);

    println!();
    Ok(())
}
