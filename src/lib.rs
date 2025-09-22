use base64::engine::general_purpose::STANDARD as base64_standard;
use base64::engine::Config;
use base64::{encoded_len, Engine};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyBufferError, PyValueError};
use pyo3::prelude::*;

fn slice_from_py_any<'a>(py_obj: &Bound<'a, PyAny>) -> PyResult<&'a [u8]> {
    let buf = PyBuffer::<u8>::get(py_obj)?;
    if !buf.is_c_contiguous() {
        return Err(PyErr::new::<pyo3::exceptions::PyBufferError, _>(
            "Object does not have a contiguous buffer",
        ));
    }

    if buf.item_size() != 1 {
        return Err(PyErr::new::<PyBufferError, _>(
            "Buffer item size is not 1 byte",
        ));
    }

    if buf.item_count() == 0 {
        return Ok(&[] as &[u8]);
    }

    let ptr = buf.buf_ptr();
    if ptr.is_null() {
        return Err(PyErr::new::<PyBufferError, _>("Buffer pointer is null"));
    }

    // SAFETY: We have verified that the buffer is contiguous and non-empty.
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, buf.len_bytes()) };

    Ok(bytes)
}

#[pyfunction]
fn b64encode(s: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    let bytes = slice_from_py_any(s)?;

    let size = encoded_len(bytes.len(), base64_standard.config().encode_padding()).map_or(
        Err(PyErr::new::<PyValueError, _>(
            "Integer overflow when calculating buffer size",
        )),
        |x| Ok(x),
    )?;

    let mut buf = vec![0u8; size];
    base64_standard
        .encode_slice(bytes, &mut buf)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Base64 encoding error: {}", e)))?;
    Ok(buf)
}

#[pymodule]
fn pyrsbase64(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(b64encode, m)?)?;
    Ok(())
}
