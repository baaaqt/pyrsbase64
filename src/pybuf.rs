use pyo3::{buffer::PyBuffer, exceptions::PyBufferError, prelude::*};
use pyo3_file::PyFileLikeObject;

pub fn convert_pybytebuf_to_slice<'py>(obj: &Bound<'py, PyAny>) -> PyResult<&'py [u8]> {
    let buf = PyBuffer::<u8>::get(obj)?;
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

pub struct PyReadableBinaryIO(PyFileLikeObject);

impl PyReadableBinaryIO {
    pub fn read(&self, py: Python<'_>, buf: &mut [u8]) -> PyResult<usize> {
        self.0.py_read(py, buf).map_err(PyErr::from)
    }
}

impl<'py> FromPyObject<'py> for PyReadableBinaryIO {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(PyReadableBinaryIO(PyFileLikeObject::py_with_requirements(
            ob.clone(),
            true,
            false,
            false,
            false,
        )?))
    }
}

pub struct PyWriteableBinaryIO(PyFileLikeObject);

impl PyWriteableBinaryIO {
    pub fn write(&self, py: Python<'_>, buf: &[u8]) -> PyResult<usize> {
        self.0.py_write(py, buf).map_err(PyErr::from)
    }
}

impl<'py> FromPyObject<'py> for PyWriteableBinaryIO {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        Ok(PyWriteableBinaryIO(PyFileLikeObject::py_with_requirements(
            ob.clone(),
            false,
            true,
            false,
            false,
        )?))
    }
}
