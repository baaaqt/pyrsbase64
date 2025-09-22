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
    ],
)
def test_encodebytes(data: "ReadableBuffer") -> None:
    data = b"hello world"
    assert base64.encodebytes(data) == pyrsbase64.encodebytes(data)
