.PHONY: build lint

lint:
	@npx -y prettier --write .
	@cargo fmt
	@cargo clippy

build: lint
	@cargo build
