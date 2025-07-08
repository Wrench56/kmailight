use tree_sitter_highlight::{HighlightConfiguration, Highlighter, HighlightEvent};
use tree_sitter_c;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_code = r#"
#include <stdio.h>

// Comment
int main() {
    int x = 42;
    printf("Hello, world! %d\n", x);
    return 0;
}
"#;

    let mut config = HighlightConfiguration::new(
        tree_sitter_c::LANGUAGE.into(),
        "c",
        tree_sitter_c::HIGHLIGHT_QUERY,
        "",
        tree_sitter_c::TAGS_QUERY
    )?;

    config.configure(&[
        "function", "type", "string", "keyword", "number", "comment", "constant", "operator", "variable"
    ]);

    let mut highlighter = Highlighter::new();
    let highlight_events = highlighter.highlight(&config, source_code.as_bytes(), None, |_| None)?;

    for event in highlight_events {
        match event? {
            HighlightEvent::Source { start, end } => {
                print!("{}", &source_code[start..end]);
            }
            HighlightEvent::HighlightStart(s) => {
                print!("{}", ansi_for_class(s.0));
            }
            HighlightEvent::HighlightEnd => {
                print!("\x1b[0m");
            }
        }
    }

    println!();
    Ok(())
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
