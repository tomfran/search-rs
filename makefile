web:
	cargo run --release --bin server ${folder}

cli:
	cargo run --release --bin search ${folder} ${action} ${min_f} ${max_p}

test:
	cargo test --release

clippy: 
	cargo clippy

clippy-pedantic:
	cargo clippy -- -W clippy::pedantic