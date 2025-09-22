use base64::engine::general_purpose::STANDARD as base64_standard;
use base64::engine::Config;
use base64::{encoded_len, Engine};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyBufferError, PyTypeError, PyValueError};
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
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, buf.item_count()) };

    Ok(bytes)
}

fn validate_altchars(altchars: &[u8]) -> PyResult<(u8, u8)> {
    if altchars.len() != 2 {
        return Err(PyErr::new::<PyValueError, _>(
            "altchars must be a bytes-like object of length 2",
        ));
    }
    Ok((altchars[0], altchars[1]))
}

#[pyfunction]
#[pyo3(signature = (s, altchars=None))]
fn b64encode(s: &Bound<'_, PyAny>, altchars: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<u8>> {
    let bytes = slice_from_py_any(s)?;
    let altchars = if let Some(alt) = altchars {
        let alt_bytes = slice_from_py_any(alt)?;
        Some(validate_altchars(alt_bytes)?)
    } else {
        None
    };

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

    if let Some(chars) = altchars {
        for byte in &mut buf {
            if *byte == b'+' {
                *byte = chars.0;
            } else if *byte == b'/' {
                *byte = chars.1;
            }
        }
    }

    Ok(buf)
}

#[pyfunction]
#[pyo3(signature = (s, altchars=None, validate=false))]
fn b64decode(
    s: &Bound<'_, PyAny>,
    altchars: Option<&Bound<'_, PyAny>>,
    validate: bool,
) -> PyResult<Vec<u8>> {
    let bytes = if let Ok(s) = s.extract::<&str>() {
        s.as_bytes()
    } else if let Ok(s) = slice_from_py_any(s) {
        s
    } else {
        return Err(PyErr::new::<PyTypeError, _>(format!(
            "argument should be a bytes-like object or ASCII string, not '{}'",
            s.get_type().name()?
        )));
    };

    let altchars = if let Some(altchars) = altchars {
        if let Ok(altchars) = altchars.extract::<&str>() {
            Some(validate_altchars(altchars.as_bytes())?)
        } else if let Ok(altchars) = slice_from_py_any(altchars) {
            Some(validate_altchars(altchars)?)
        } else {
            return Err(PyErr::new::<PyTypeError, _>(format!(
                "altchars should be a bytes-like object or ASCII string, not '{}'",
                altchars.get_type().name()?
            )));
        }
    } else {
        None
    };

    let bytes = if let Some((c1, c2)) = altchars {
        if !validate {
            bytes
                .iter()
                .filter_map(|&b| {
                    if b == c1 {
                        Some(b'+')
                    } else if b == c2 {
                        Some(b'/')
                    } else {
                        if b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=' {
                            Some(b)
                        } else {
                            None
                        }
                    }
                })
                .collect()
        } else {
            bytes
                .iter()
                .map(|&b| {
                    if b == c1 {
                        b'+'
                    } else if b == c2 {
                        b'/'
                    } else {
                        b
                    }
                })
                .collect()
        }
    } else if !validate {
        bytes
            .iter()
            .filter(|&&b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=')
            .cloned()
            .collect()
    } else {
        bytes.to_vec()
    };

    base64_standard
        .decode(&bytes)
        .map_err(|e: base64::DecodeError| {
            PyErr::new::<PyValueError, _>(format!("Base64 decoding error: {}", e))
        })
}

#[pymodule]
fn pyrsbase64(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(b64encode, m)?)?;
    m.add_function(wrap_pyfunction!(b64decode, m)?)?;
    Ok(())
}
