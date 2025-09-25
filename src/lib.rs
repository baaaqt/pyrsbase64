use base64::alphabet::Alphabet;
use base64::engine::general_purpose::{GeneralPurpose, PAD, STANDARD as base64_standard};
use base64::engine::Config;
use base64::{encoded_len, Engine};
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use crate::altchars::Altchars;
use crate::pybuf::convert_pybytebuf_to_slice;

mod altchars;
mod pybuf;

fn altchars_engine(altchars: Altchars) -> PyResult<GeneralPurpose> {
    if altchars == Altchars::default() {
        return Ok(base64_standard);
    }
    let alphabet = Alphabet::new(&format!(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789{}",
        altchars
    ))
    .map_err(|_| PyErr::new::<PyValueError, _>(format!("Invalid altchars: {}", altchars)))?;
    Ok(GeneralPurpose::new(&alphabet, PAD))
}

#[pyfunction]
#[pyo3(signature = (s, altchars=None))]
fn b64encode(s: &Bound<'_, PyAny>, altchars: Option<Altchars>) -> PyResult<Vec<u8>> {
    let bytes = convert_pybytebuf_to_slice(s)?;
    let engine = altchars_engine(altchars.unwrap_or(Altchars::default()))?;

    let size = encoded_len(bytes.len(), base64_standard.config().encode_padding()).map_or(
        Err(PyErr::new::<PyValueError, _>(
            "Integer overflow when calculating buffer size",
        )),
        |x| Ok(x),
    )?;

    let mut buf = vec![0u8; size];
    engine.encode_slice(bytes, &mut buf).unwrap();

    Ok(buf)
}

#[pyfunction]
#[pyo3(signature = (s, altchars=None, validate=false))]
fn b64decode(
    s: &Bound<'_, PyAny>,
    altchars: Option<Altchars>,
    validate: bool,
) -> PyResult<Vec<u8>> {
    let bytes = if let Ok(s) = s.extract::<&str>() {
        s.as_bytes()
    } else if let Ok(s) = convert_pybytebuf_to_slice(s) {
        s
    } else {
        return Err(PyErr::new::<PyTypeError, _>(format!(
            "argument should be a bytes-like object or ASCII string, not '{}'",
            s.get_type().name()?
        )));
    };

    let (altchars, engine) = if let Some(altchars) = altchars {
        let engine = altchars_engine(altchars)?;
        (altchars, engine)
    } else {
        (Altchars::default(), base64_standard)
    };

    let decode_result = if !validate {
        let mut valid = [false; 256];

        for &b in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789=" {
            valid[b as usize] = true;
        }
        valid[altchars.plus() as usize] = true;
        valid[altchars.slash() as usize] = true;

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
