use pyo3::prelude::*;
use tree_sitter_highlight::{HighlightConfiguration, Highlighter};
use pyo3::exceptions::PyValueError;

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

/// Formats the sum of two numbers as string.
#[pyfunction]
fn highlight(string: &str) -> PyResult<i32> {
    let mut hl = Highlighter::new();
    let py_lang = tree_sitter_python::LANGUAGE;
    let mut py_config = HighlightConfiguration::new(
        py_lang.into(),
        "python",
        tree_sitter_python::HIGHLIGHTS_QUERY,
        "",
        tree_sitter_python::TAGS_QUERY,
    );

    py_config.configure(&HIGHLIGHT_NAMES);

    let highlights = hl.highlight(&py_config, string.as_bytes(), None, |_| None) else ;

    Ok(2)
}

/// A Python module implemented in Rust.
#[pymodule]
fn a(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(highlight, m)?)?;
    Ok(())
}
