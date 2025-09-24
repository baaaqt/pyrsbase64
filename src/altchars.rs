use std::fmt::Display;

use pyo3::prelude::*;

use crate::pybuf::convert_pybytebuf_to_slice;

/// A pair of bytes representing the alternative characters for
/// padding and the '/' character in base64 encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Altchars((u8, u8));

impl Altchars {
    #[inline]
    pub fn new(plus: u8, slash: u8) -> Self {
        Altchars((plus, slash))
    }

    /// Returns the default altchars for standard base64 encoding.
    /// This is equivalent to `Altchars::new(b'+', b'/')`.
    pub const fn default() -> Self {
        Altchars((b'+', b'/'))
    }

    /// Returns the '+' alternative character.
    #[inline]
    pub fn plus(&self) -> u8 {
        self.0 .0
    }

    /// Returns the '/' alternative character.
    #[inline]
    pub fn slash(&self) -> u8 {
        self.0 .1
    }
}

impl<'py> FromPyObject<'py> for Altchars {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let bytes = convert_pybytebuf_to_slice(ob)?;
        if bytes.len() != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "altchars must be exactly 2 bytes long",
            ));
        }
        Ok(Altchars::new(bytes[0], bytes[1]))
    }
}

impl Display for Altchars {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0 .0 as char, self.0 .1 as char)
    }
}
