use pyo3::exceptions::PyException;
use pyo3::prelude::*;

pyo3::create_exception!(_native, OpenPxError, PyException);
pyo3::create_exception!(_native, NetworkError, OpenPxError);
pyo3::create_exception!(_native, ExchangeError, OpenPxError);
pyo3::create_exception!(_native, AuthenticationError, ExchangeError);

pub fn register_exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("OpenPxError", m.py().get_type::<OpenPxError>())?;
    m.add("NetworkError", m.py().get_type::<NetworkError>())?;
    m.add("ExchangeError", m.py().get_type::<ExchangeError>())?;
    m.add(
        "AuthenticationError",
        m.py().get_type::<AuthenticationError>(),
    )?;
    Ok(())
}

pub fn to_py_err(msg: String) -> PyErr {
    // Map error message patterns to specific exception types
    if msg.contains("Authentication") || msg.contains("auth") {
        AuthenticationError::new_err(msg)
    } else if msg.contains("Network") || msg.contains("Http") || msg.contains("Timeout") {
        NetworkError::new_err(msg)
    } else if msg.contains("Exchange") {
        ExchangeError::new_err(msg)
    } else {
        OpenPxError::new_err(msg)
    }
}
