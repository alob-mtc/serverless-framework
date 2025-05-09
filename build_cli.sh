#!/bin/sh
set -eu pipefail
pushd cli
cargo build --release
popd

sudo chmod +x target/release/invok 
sudo cp target/release/invok /usr/local/bin
