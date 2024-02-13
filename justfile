export RUSTFLAGS := "-D warnings"

default:
    just --list

fmt:
    cargo +nightly-2023-05-08 fmt

fmt-check:
    cargo +nightly-2023-05-08 fmt --check

clippy-workspace:
    cargo clippy --examples --tests --all-features

clippy-aries-vcx features:
    cargo clippy -p aries_vcx --features legacy_proof --features {{features}} --no-default-features

clippy-aries-vcx-core features:
    cargo clippy -p aries_vcx_core --features legacy_proof,vdrtools_wallet --features {{features}}

check-workspace:
    cargo check --tests --all-features

check-aries-vcx:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" --all-features --tests

check-aries-vcx-anoncreds:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdrtools_wallet,anoncreds --tests

check-aries-vcx-credx:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdrtools_wallet,credx --tests

check-aries-vcx-core:
    cargo test --manifest-path="aries/aries_vcx_core/Cargo.toml" --all-features --tests

check-aries-vcx-core-anoncreds:
    cargo test --manifest-path="aries/aries_vcx_core/Cargo.toml" -F vdrtools_wallet,anoncreds --tests

check-aries-vcx-core-credx:
    cargo test --manifest-path="aries/aries_vcx_core/Cargo.toml" -F vdrtools_wallet,credx --tests

test-unit:
    RUST_TEST_THREADS=1 cargo test --workspace --lib --exclude aries-vcx-agent --exclude libvdrtools --exclude wallet_migrator --exclude mediator

test-integration-aries-vcx:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdrtools_wallet,credx -- --ignored

test-integration-aries-vcx-anoncreds-rs:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F anoncreds --test test_revocations --test test_proof_presentation --test test_anoncreds -- --ignored

test-integration-aries-vcx-mysql:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" test_mysql -- --include-ignored

test-integration-aries-vcx-vdrproxy:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdr_proxy_ledger,credx -- --ignored

test-integration-libvcx:
    RUST_TEST_THREADS=1 cargo test --manifest-path="aries/misc/legacy/libvcx_core/Cargo.toml" -- --include-ignored

test-integration-did-crate:
    cargo test --examples -p did_doc -p did_parser -p did_resolver -p did_resolver_registry -p did_resolver_sov -p did_resolver_web -p did_doc_sov -p did_key -p did_peer --test "*"
