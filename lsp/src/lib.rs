mod lsclient;
mod pywrapper;

use pyo3::prelude::*;

#[pymodule]
fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<pywrapper::LsClient>()?;
    Ok(())
}
