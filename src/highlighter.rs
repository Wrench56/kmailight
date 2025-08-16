use std::collections::HashMap;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

pub struct HighlighterEngine {
    configs: HashMap<&'static str, HighlightConfiguration>,
    highlighter: Highlighter,
}

impl HighlighterEngine {
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        let mut c_config = HighlightConfiguration::new(
            tree_sitter_c::LANGUAGE.into(),
            "c",
            tree_sitter_c::HIGHLIGHT_QUERY,
            "",
            tree_sitter_c::TAGS_QUERY,
        )
        .unwrap();
        c_config.configure(&[
            "function", "type", "string", "keyword", "number", "comment", "constant", "operator",
            "variable",
        ]);
        configs.insert("c", c_config);

        Self {
            configs,
            highlighter: Highlighter::new(),
        }
    }

    pub fn highlight(&mut self, lang: &str, code: &str) -> String {
        let Some(config) = self.configs.get(lang) else {
            return code.to_string();
        };

        let events = self
            .highlighter
            .highlight(config, code.as_bytes(), None, |_| None)
            .unwrap();

        let mut output = String::new();
        for event in events {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    output.push_str(&code[start..end]);
                }
                HighlightEvent::HighlightStart(s) => {
                    output.push_str(ansi_for_class(s.0));
                }
                HighlightEvent::HighlightEnd => {
                    output.push_str("\x1b[0m");
                }
            }
        }
        output
    }

    /// Highlight an individual hunk of text
    ///
    /// The only thing highlighted for now are the quoting marks (">")
    pub fn highlight_text(&mut self, text: &str) -> String {
        const BLUE: &str = "\x1b[34m";
        const RESET: &str = "\x1b[0m";

        fn paint_line(line: &str) -> String {
            let mut out = String::with_capacity(line.len() + 8);
            let mut started = false;

            for (idx, ch) in line.char_indices() {
                match ch {
                    ' ' | '\t' if !started => {
                        out.push(ch);
                    }
                    '>' if !started => {
                        out.push_str(BLUE);
                        out.push('>');
                        out.push_str(RESET);
                    }
                    _ => {
                        out.push_str(&line[idx..]);
                        return out;
                    }
                }
                if ch != ' ' && ch != '\t' {
                    started = true;
                }
            }
            out
        }

        let ends_with_nl = text.ends_with('\n');
        let mut result = text.lines().map(paint_line).collect::<Vec<_>>().join("\n");

        if ends_with_nl {
            result.push('\n');
        }
        result
    }
}

/// Convert highlight class ID to ANSI color
pub fn ansi_for_class(class: usize) -> &'static str {
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
