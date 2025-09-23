use base64::alphabet::Alphabet;
use base64::engine::general_purpose::{GeneralPurpose, PAD, STANDARD as base64_standard};
use base64::engine::Config;
use base64::{encoded_len, Engine};
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyBufferError, PyTypeError, PyValueError};
use pyo3::prelude::*;

fn altchars_engine(altchars: [u8; 2]) -> PyResult<GeneralPurpose> {
    let altchars = altchars.iter().map(|&x| x as char).collect::<String>();
    let alphabet = Alphabet::new(&format!(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789{}",
        altchars
    ))
    .map_err(|_| PyErr::new::<PyValueError, _>(format!("Invalid altchars: {}", altchars)))?;
    Ok(GeneralPurpose::new(&alphabet, PAD))
}

fn get_engine(altchars: Option<&Bound<'_, PyAny>>) -> PyResult<([u8; 2], GeneralPurpose)> {
    if let Some(alt) = altchars {
        let alt_bytes = slice_from_py_any(alt)?;
        let alt_bytes = validate_altchars(alt_bytes)?;
        let engine = altchars_engine(alt_bytes)?;
        Ok((alt_bytes, engine))
    } else {
        Ok(([b'+', b'/'], base64_standard))
    }
}

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

fn validate_altchars(altchars: &[u8]) -> PyResult<[u8; 2]> {
    if altchars.len() != 2 {
        return Err(PyErr::new::<PyValueError, _>(
            "altchars must be a bytes-like object of length 2",
        ));
    }
    Ok([altchars[0], altchars[1]])
}

#[pyfunction]
#[pyo3(signature = (s, altchars=None))]
fn b64encode(s: &Bound<'_, PyAny>, altchars: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<u8>> {
    let bytes = slice_from_py_any(s)?;
    let (_, engine) = get_engine(altchars)?;

    let size = encoded_len(bytes.len(), base64_standard.config().encode_padding()).map_or(
        Err(PyErr::new::<PyValueError, _>(
            "Integer overflow when calculating buffer size",
        )),
        |x| Ok(x),
    )?;

    let mut buf = vec![0u8; size];
    engine
        .encode_slice(bytes, &mut buf)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Base64 encoding error: {}", e)))?;

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

    let (altchars, engine) = get_engine(altchars)?;

    let decode_result = if !validate {
        let mut valid = [false; 256];

        for &b in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789=" {
            valid[b as usize] = true;
        }
        valid[altchars[0] as usize] = true;
        valid[altchars[1] as usize] = true;

        let bytes = bytes
            .iter()
            .copied()
            .filter(|&b| valid[b as usize])
            .collect::<Vec<u8>>();
        engine.decode(&bytes)
    } else {
        engine.decode(bytes)
    };

    decode_result.map_err(|e: base64::DecodeError| {
        PyErr::new::<PyValueError, _>(format!("Base64 decoding error: {}", e))
    })
}

#[pymodule]
fn pyrsbase64(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(b64encode, m)?)?;
    m.add_function(wrap_pyfunction!(b64decode, m)?)?;
    Ok(())
}
