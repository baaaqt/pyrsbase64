test:
	maturin develop
	uv run pytest

test-release:
	maturin develop -r
	uv run pytest

benchmark:
	maturin develop -r
	uv run bench/bench.py
