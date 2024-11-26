default:
    just --list

fmt:
    cargo +nightly-2023-05-08 fmt

fmt-check:
    cargo +nightly-2023-05-08 fmt --check

clippy-workspace wallet:
    cargo clippy --examples --tests --no-default-features -F anoncreds,vdr_proxy_ledger,legacy_proof,{{wallet}}

clippy-aries-vcx features:
    cargo clippy -p aries_vcx --features legacy_proof --features {{features}} --no-default-features

check-workspace:
    cargo check --tests --all-features

check-aries-vcx-anoncreds:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F askar_wallet,anoncreds --tests

test-unit test_name="":
    RUST_TEST_THREADS=1 cargo test --workspace --lib --exclude aries-vcx-agent --exclude mediator {{test_name}} -F did_doc/jwk -F public_key/jwk

test-integration-aries-vcx features test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F {{features}} -- --ignored {{test_name}}

test-integration-aries-vcx-vdrproxy test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdr_proxy_ledger,anoncreds -- --ignored {{test_name}}

test-integration-did-crate test_name="":
    cargo test --examples -p did_doc -p did_parser_nom -p did_resolver -p did_resolver_registry -p did_resolver_sov -p did_resolver_web -p did_key -p did_peer -p did_jwk -F did_doc/jwk --test "*"
