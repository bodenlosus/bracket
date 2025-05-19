use pyo3::prelude::*;
mod hl;

#[pymodule(name = "highlighter")]
fn highlighter(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<hl::HL>()?;
    Ok(())
}
