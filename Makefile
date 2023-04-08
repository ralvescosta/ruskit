install-tools:
	@cargo install cargo-tarpaulin
	@cargo install cargo-audit --features=fix
	@cargo install cargo-udeps --locked
	@cargo install cargo-bloat

audit:
	cargo audit

tests:
	cargo +nightly tarpaulin --workspace --timeout 120 --out Xml

bloat-fn: # Get a list of the biggest functions in the release build
	cargo bloat --release -n 10

bloat-crate: #Get a list of the biggest dependencies in the release build:
	cargo bloat --release --crates

