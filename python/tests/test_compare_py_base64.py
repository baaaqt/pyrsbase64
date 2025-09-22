import base64
from typing import TYPE_CHECKING

import pyrsbase64
import pytest

if TYPE_CHECKING:
    from _typeshed import ReadableBuffer


@pytest.mark.parametrize(
    "data",
    [
        b"",
        b"hello world",
        b"hello world!" * 100,
        bytes(range(256)),
        b"\x00" * 1024,
        memoryview(b"hello world"),
        bytearray(b"hello world"),
    ],
    ids=lambda x: f"data_len={len(x)}",
)
def test_b64encode(data: "ReadableBuffer") -> None:
    data = b"hello world"
    assert base64.b64encode(data) == pyrsbase64.b64encode(data)


@pytest.mark.parametrize(
    "data",
    [
        b"\xff\xe0\x00",
        b"hello world\xff\xe0\x00",
        b"hello world!" * 100 + b"\xff\xe0\x00",
        b"\xff\xe0\x00" * 256,
    ],
    ids=lambda x: f"data_len={len(x)}",
)
def test_b64encode_altchars(data: "ReadableBuffer") -> None:
    altchars = b"-_"
    assert base64.b64encode(data, altchars) == pyrsbase64.b64encode(data, altchars)


@pytest.mark.parametrize(
    "data",
    [
        b"",
        b"hello world",
        b"hello world!" * 100,
        bytes(range(256)),
        b"\x00" * 1024,
        memoryview(b"hello world"),
        bytearray(b"hello world"),
    ],
    ids=lambda x: f"data_len={len(x)}",
)
def test_b64decode(data: "ReadableBuffer") -> None:
    encoded = base64.b64encode(data)
    assert base64.b64decode(encoded) == pyrsbase64.b64decode(encoded)


@pytest.mark.parametrize(
    "data",
    [
        b"\xff\xe0\x00",
        b"hello world\xff\xe0\x00",
        b"hello world!" * 100 + b"\xff\xe0\x00",
        b"\xff\xe0\x00" * 256,
    ],
    ids=lambda x: f"data_len={len(x)}",
)
def test_b64decode_altchars(data: "ReadableBuffer") -> None:
    altchars = b"-_"
    encoded = base64.b64encode(data, altchars)
    assert base64.b64decode(encoded, altchars) == pyrsbase64.b64decode(
        encoded, altchars
    )
