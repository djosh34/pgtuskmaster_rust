check:
	cargo check --all-targets

test:
	cargo test --all-targets

test-bdd:
	cargo test --test bdd_state_watch

lint:
	cargo clippy --all-targets --all-features -- -D warnings
