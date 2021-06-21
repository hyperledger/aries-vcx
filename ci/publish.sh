#!/bin/bash

cd "$TOML_PATH_AGENCY_DIR" && cargo publish --allow-dirty --dry-run
cd "$TOML_PATH_ARIES_VCX_DIR" && cargo publish --allow-dirty --dry-run