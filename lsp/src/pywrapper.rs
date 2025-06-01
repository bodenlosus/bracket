use crate::lsclient;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::sync::{Arc, RwLock};
use tokio::{sync::futures, try_join};

#[pyclass]
pub struct LsClient {
    client: Arc<RwLock<lsclient::LspClient>>,
}

#[pymethods]
impl LsClient {
    #[new]
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(lsclient::LspClient::new())),
        }
    }

    pub fn start(&self) -> PyResult<()> {
        let Ok(mut client) = self.client.try_write() else {
            return Err(PyErr::new::<PyRuntimeError, _>(
                "Could not acquire read write lock",
            ));
        };
        let start_fut = client.start("pylsp", &[]);
        let res_fut = client.handle_response();

        try_join!(start_fut, res_fut);
        //     .map_err(|e| {
        //     PyErr::new::<PyRuntimeError, _>(format!("Lsp client start failed: {e:?}"))
        // })?;

        Ok(())
    }
    pub fn request_stop(&self) -> PyResult<()> {
        let Ok(mut client) = self.client.try_write() else {
            return Err(PyErr::new::<PyRuntimeError, _>(
                "Could not acquire read write lock",
            ));
        };
        client.shutdown();
        Ok(())
    }
}
