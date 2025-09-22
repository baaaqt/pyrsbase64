use base64::engine::general_purpose::STANDARD as base64_standard;
use base64::engine::Config;
use base64::{encoded_len, Engine};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn encodebytes(s: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    let buf = PyBuffer::<u8>::get(s)?;
    if !buf.is_c_contiguous() {
        return Err(PyErr::new::<pyo3::exceptions::PyBufferError, _>(
            "Object does not have a contiguous buffer",
        ));
    }

    if buf.item_count() == 0 {
        return Ok(vec![]);
    }

    let ptr = buf.buf_ptr();
    if ptr.is_null() {
        return Err(PyErr::new::<pyo3::exceptions::PyBufferError, _>(
            "Buffer pointer is null",
        ));
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, buf.len_bytes()) };

    let size = encoded_len(bytes.len(), base64_standard.config().encode_padding()).map_or(
        Err(PyErr::new::<PyValueError, _>(
            "Integer overflow when calculating buffer size",
        )),
        |x| Ok(x + 1),
    )?;

    let mut buf = vec![0u8; size];
    base64_standard
        .encode_slice(bytes, &mut buf)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Base64 encoding error: {}", e)))?;
    buf[size - 1] = b'\n';
    Ok(buf)
}

#[pymodule]
fn pyrsbase64(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(encodebytes, m)?)?;
    Ok(())
}
