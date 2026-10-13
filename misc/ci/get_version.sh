#!/bin/sh
# Prints the version from the [package] section of the root Cargo.toml, which is the single source of truth.
awk -F'"' '/^\[package\]/ { p = 1; next } /^\[/ { p = 0 } p && /^version *=/ { print $2; exit }' \
  "$(dirname "$0")/../../Cargo.toml"
