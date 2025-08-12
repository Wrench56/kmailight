use std::env;
use std::fs;
use std::io::{self, Read};

use crate::parser::line::Line;

mod debug;
mod highlighter;

mod parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_code = if let Some(path) = env::args().nth(1) {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let highlighter = highlighter::HighlighterEngine::new();

    let lines = Line::parse_lines(&source_code);
    debug::print_lines(&lines);

    let spans = parser::span::build_spans(&lines);
    debug::print_spans(&spans);

    println!();
    Ok(())
}
