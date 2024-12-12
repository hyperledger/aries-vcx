default:
    just --list

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

clippy:
    cargo clippy --examples --tests --all-features

# The following need review:
check-workspace:
    cargo check --tests --all-features

check-aries-vcx-anoncreds:
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F askar_wallet,anoncreds --tests

test-unit test_name="":
    RUST_TEST_THREADS=1 cargo test --workspace --lib --exclude aries-vcx-agent --exclude mediator {{test_name}} -F did_doc/jwk -F public_key/jwk -F aries_vcx_ledger/cheqd

test-integration-aries-vcx features test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F {{features}} -- --ignored {{test_name}}

test-integration-aries-vcx-vdrproxy test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdr_proxy_ledger,anoncreds -- --ignored {{test_name}}

test-integration-aries-vcx-ledger:
    cargo test --manifest-path="aries/aries_vcx_ledger/Cargo.toml" -F cheqd

test-integration-did-crate test_name="":
    cargo test --examples -p did_doc -p did_parser_nom -p did_resolver -p did_resolver_registry -p did_resolver_sov -p did_resolver_web -p did_key -p did_peer -p did_jwk -p did_cheqd -F did_doc/jwk --test "*"