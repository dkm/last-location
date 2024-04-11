#!/usr/bin/env bash
. "$HOME/.cargo/env"

set -x

cargo test --verbose
