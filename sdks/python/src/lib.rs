mod error;
mod exchange;
mod stream;
mod websocket;

use pyo3::prelude::*;

/// Dedicated tokio runtime for all async operations.
/// Created once on module import, shared across all exchange instances.
fn get_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime")
    })
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<exchange::NativeExchange>()?;
    m.add_class::<websocket::NativeWebSocket>()?;
    m.add_class::<stream::NativeOrderbookStream>()?;
    m.add_class::<stream::NativeActivityStream>()?;
    error::register_exceptions(m)?;
    Ok(())
}
