#!/usr/bin/env bash

echo "*** Initializing becnhmarking"

PALLET="fruniques"
BASE_PATH="/home/armando/Desktop/hashed-pallets"

OUTPUT_PATH="$BASE_PATH/pallets/$PALLET/src/weights.rs"
TEMPLATE_PATH="$BASE_PATH/.maintain/frame-weight-template.hbs"

cargo build --package hashed-parachain --release --features runtime-benchmarks

./target/release/hashed-parachain benchmark pallet \
--chain dev \
--pallet "pallet_$PALLET" \
--wasm-execution=compiled \
--extrinsic '*' \
--steps 50 \
--repeat 20 \
--output "$OUTPUT_PATH" \
--template "$TEMPLATE_PATH" \
