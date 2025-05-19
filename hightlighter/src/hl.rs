use pyo3::{exceptions::PyOSError, prelude::*};
use std::{
    cell::{Cell, RefCell},
    error::Error,
    sync::RwLock,
};
use tree_sitter_highlight::{HighlightConfiguration, Highlighter};

const HIGHLIGHT_NAMES: [&str; 26] = [
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "keyword",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

#[pyclass]
pub struct HL {
    highlighter: RwLock<Highlighter>,
    recognized_names: Vec<String>,
    configuration: RwLock<Option<HighlightConfiguration>>,
}
#[derive(Debug)]
struct HighlighterError {
    message: String,
}

impl HighlighterError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl Error for HighlighterError {}

impl std::fmt::Display for HighlighterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error doing Syntax Highlighting {}", self.message)
    }
}

// impl std::convert::From<HighlighterError> for PyErr {
//     fn from(value: HighlighterError) -> Self {
//         PyOSError::new_err(value.to_string())
//     }
// }

impl std::convert::Into<PyErr> for HighlighterError {
    fn into(self) -> PyErr {
        PyOSError::new_err(self.to_string())
    }
}

#[pymethods]
impl HL {
    #[new]
    pub fn new() -> Self {
        Self {
            highlighter: RwLock::new(Highlighter::new()),
            recognized_names: Vec::new(),
            configuration: RwLock::new(None),
        }
    }
    pub fn set_language(&self) -> PyResult<()> {
        let lang = tree_sitter_python::LANGUAGE;
        let mut config = match HighlightConfiguration::new(
            lang.into(),
            "python",
            tree_sitter_python::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_python::TAGS_QUERY,
        ) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{e}");
                return Err(
                    HighlighterError::new("Could not create Highlighter Function".into()).into(),
                );
            }
        };

        config.configure(&self.recognized_names);

        let Ok(mut c) = self.configuration.try_write() else {
            return Err(HighlighterError::new("Could not lock on config".into()).into());
        };
        *c = Some(config);

        Ok(())
    }
    pub fn highlight(&self, string: &str) -> PyResult<()> {
        println!("Input: {string}");
        let string = string.trim_start().trim_end();
        let config = self.configuration.try_read();
        let Ok(config) = config.as_ref() else {
            return Err(HighlighterError::new("Could not lock on config".into()).into());
        };

        let Some(config) = config.as_ref() else {
            return Err(HighlighterError::new("Could not lock on config".into()).into());
        };

        let Ok(mut highligher) = self.highlighter.try_write() else {
            return Err(HighlighterError::new("Could not lock on highlighter".into()).into());
        };

        let highlights = match highligher.highlight(config, string.as_bytes(), None, |_| None) {
            Ok(hl) => hl,
            Err(e) => {
                eprintln!("{e}");
                return Err(HighlighterError::new("Error while highlighting".into()).into());
            }
        };

        for event in highlights {
            let Ok(event) = event else {
                continue;
            };

            match event {
                tree_sitter_highlight::HighlightEvent::Source { start, end } => {
                    let s = &string[start..end].replace("\n", "\\n");
                    println!("String: {s}");
                    println!("{start} - {end} ")
                }
                tree_sitter_highlight::HighlightEvent::HighlightStart(highlight) => {
                    let hl_type = HIGHLIGHT_NAMES.get(highlight.0);
                    if let Some(hl_type) = hl_type {
                        println!("Name: {hl_type}");
                    }
                }
                tree_sitter_highlight::HighlightEvent::HighlightEnd => println!("style ended \n"),
            }
        }

        Ok(())
    }
}

/// A Python module implemented in Rust.
const CODE: &str = "

def hello(s:str):
    print(s + \"Hello\")
    return s

if __name__ == \"__main__\":
    hello()
";
