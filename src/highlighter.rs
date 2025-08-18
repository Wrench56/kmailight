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

    /// Highlight an individual hunk of code
    pub fn highlight_code(&mut self, lang: &str, code: &str) -> String {
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

    /// Helper function to highlight quoting marks
    fn highlight_quoting_marks(&mut self, text: &str) -> String {
        const BLUE: &str = "\x1b[34m";
        const RESET: &str = "\x1b[0m";

        /// Paint only the quote marks at the beginning of the quoted lines
        #[cfg(not(feature = "quote-paint-full"))]
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

        /// Paint the full quoted lines
        #[cfg(feature = "quote-paint-full")]
        fn paint_line(line: &str) -> String {
            const BLUE: &str = "\x1b[34m";
            const RESET: &str = "\x1b[0m";

            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            let mut j = i;
            while j < bytes.len() && bytes[j] == b'>' {
                j += 1;
            }

            if j == i {
                return line.to_string();
            }

            let mut out = String::with_capacity(line.len() + BLUE.len() + RESET.len());
            out.push_str(BLUE);
            out.push_str(line);
            out.push_str(RESET);
            out
        }

        let ends_with_nl = text.ends_with('\n');
        let mut result = text.lines().map(paint_line).collect::<Vec<_>>().join("\n");

        if ends_with_nl {
            result.push('\n');
        }
        result
    }

    /// Highlight an individual hunk of text
    ///
    /// The only thing highlighted for now are the quoting marks (">")
    pub fn highlight_text(&mut self, text: &str) -> String {
        self.highlight_quoting_marks(text)
    }

    /// Highlight a diffheader
    pub fn highlight_diffh(&mut self, diffh: &str) -> String {
        self.highlight_quoting_marks(diffh)
    }

    /// Highlight an individual hunk of text
    pub fn highlight_diffm(&mut self, diffm: &str) -> String {
        self.highlight_quoting_marks(diffm)
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
