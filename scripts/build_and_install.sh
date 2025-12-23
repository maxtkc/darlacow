#!/bin/bash

# If anything exits with a non-zero status, exit immediately.
set -e

# Build the code. This should make a target/debug/ directory that has the darlacow binary in it.
echo "Building darlacow code..."
cargo build
echo "... code has been successfully built."

# Copy the binary to the main darlacow directory.
echo "Installing darlacow binary..."
cp target/debug/darlacow ./darlacow
echo "... binary has been successfully installed."
