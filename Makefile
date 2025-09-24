lint:
	maturin develop
	uv run ruff check python --fix
	uv run mypy python

format:
	uv run ruff format python

test:
	maturin develop
	uv run pytest

test-r:
	maturin develop -r
	uv run pytest

benchmark:
	maturin develop -r
	uv run bench/bench.py
