mod errors;
mod lsclient;
mod parse;
mod pywrapper;
mod utils;

use pyo3::prelude::*;

#[pymodule]
fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<pywrapper::LsClient>()?;
    Ok(())
}
