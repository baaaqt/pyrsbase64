use pyo3::{buffer::PyBuffer, exceptions::PyBufferError, prelude::*};

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
