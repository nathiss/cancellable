lint-check:
	cargo check

lint-clippy:
ifeq (, $(shell which cargo-clippy))
	@echo "cargo-clippy is not installed."
	@echo "Install with:"
	@echo "    cargo install cargo-clippy"
else
	cargo clippy
endif

lint-outdated:
ifeq (, $(shell which cargo-outdated))
	@echo "cargo-outdated is not installed."
	@echo "Install with:"
	@echo "    cargo install cargo-outdated"
else
	cargo outdated
endif

list-udeps:
ifeq (, $(shell which cargo-udeps))
	@echo "cargo-udeps is not installed."
	@echo "Install with:"
	@echo "    cargo install cargo-udeps"
else
	cargo +nightly udeps
endif

lint: lint-check lint-clippy lint-outdated list-udeps
