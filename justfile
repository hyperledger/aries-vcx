default:
    just --list

fmt:
    cargo +nightly-2023-05-08 fmt

fmt-check:
    cargo +nightly-2023-05-08 fmt --check

clippy-workspace wallet:
    cargo clippy --examples --tests --no-default-features -F credx,anoncreds,vdr_proxy_ledger,legacy_proof,{{wallet}}

clippy-aries-vcx features:
    cargo clippy -p aries_vcx --features legacy_proof,vdrtools_wallet --features {{features}} --no-default-features

clippy-aries-vcx-core features:
    cargo clippy -p aries_vcx_core --features legacy_proof,vdrtools_wallet --features {{features}}

check-workspace:
    cargo check --tests --all-features

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

test-unit test_name="":
    RUST_TEST_THREADS=1 cargo test --workspace --lib --exclude aries-vcx-agent --exclude libvdrtools --exclude wallet_migrator --exclude aries_vcx_core {{test_name}} -F vdrtools_wallet

test-integration-aries-vcx-core features:
    cargo test --manifest-path="aries/aries_vcx_core/Cargo.toml" -F {{features}}

test-compatibility-aries-vcx-core:
    cargo test --manifest-path="aries/aries_vcx_core/Cargo.toml" -F vdrtools_wallet,askar_wallet wallet_compatibility_

test-wallet-migrator:
    cargo test --manifest-path="aries/misc/wallet_migrator/Cargo.toml" -F vdrtools_wallet,askar_wallet

test-integration-aries-vcx test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" --no-default-features -F credx,vdrtools_wallet -- --ignored {{test_name}}

test-integration-aries-vcx-anoncreds-rs test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F anoncreds,vdrtools_wallet --test test_revocations --test test_proof_presentation --test test_anoncreds --test test_verifier -- --ignored {{test_name}}

test-integration-aries-vcx-mysql test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdrtools_wallet test_mysql -- --include-ignored {{test_name}}

test-integration-aries-vcx-vdrproxy test_name="":
    cargo test --manifest-path="aries/aries_vcx/Cargo.toml" -F vdr_proxy_ledger,credx,vdrtools_wallet -- --ignored {{test_name}}

