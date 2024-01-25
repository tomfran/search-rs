web:
	cargo run --release --bin server $(index_name)

cli:
	cargo run --release --bin search $(index_name) ${action}

test:
	cargo test --release
