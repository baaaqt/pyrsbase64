lint:
	maturin develop
	ruff check python --fix
	mypy python

format:
	ruff format python

test:
	maturin develop
	uv run pytest

test-release:
	maturin develop -r
	uv run pytest

benchmark:
	maturin develop -r
	uv run bench/bench.py
