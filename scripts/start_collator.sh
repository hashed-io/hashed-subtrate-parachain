#!/usr/bin/env bash

# Run locally
# ./target/release/hashed-parachain --collator --base-path /tmp/chain-170223/collator-data/ --force-authoring --port 40333 --rpc-port 9946 --rpc-external --rpc-cors all --rpc-methods unsafe --chain hashed --  --execution wasm  --base-path /tmp/chain-170223/relay-data/ --chain ./resources/polkadot.json --port 30333 --rpc-port 9944 --rpc-external --rpc-cors all --rpc-methods unsafe --wasm-execution Compiled --pruning 10000
# ./target/release/hashed-parachain --collator --base-path /tmp/chain-170223/collator-data/ --force-authoring --port 40333 --rpc-port 9946 --rpc-external --rpc-cors all --rpc-methods unsafe --chain /home/sebastian/Downloads/hashed-chain-spec-raw-170223.json --  --execution wasm  --base-path /tmp/chain-170223/relay-data/ --chain ./resources/polkadot.json --port 30333 --rpc-port 9944 --rpc-external --rpc-cors all --rpc-methods unsafe --wasm-execution Compiled --pruning 1000

usage="./start_collator.sh [hashed|luhn|md5] <base-data-path>"
if [ $# -ne 2 ]; then
    echo $usage
    exit 1
fi

if [[ ($1 != 'hashed' && $1 != 'luhn' && $1 != 'md5') ]]; then
    echo $usage
    exit 1
fi

relay_chain="polkadot"

if [[ $1 == 'luhn' ]]; then
    relay_chain="kusama"
fi

if [[ $1 == 'md5' ]]; then
    relay_chain="rococo"
fi

SCRIPT_PATH="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
BASE_PATH="$(dirname "${SCRIPT_PATH}")"
RESOURCES_PATH="$BASE_PATH/resources"
#cargo build --release

collator_args=(
    --collator
    --base-path $2/collator-data/
    --force-authoring
    --rpc-port 9946
    --rpc-external
    --rpc-cors all
    --chain $RESOURCES_PATH/$1-collator-raw-spec.json
)
if [[ ! -z ${NODEKEY} ]]; then
    collator_args+=(--node-key ${NODEKEY})
fi

if [[ ! -z ${BOOTNODES} ]]; then
    collator_args+=(--bootnodes ${BOOTNODES})
fi

if [[ ! -z ${PUBLIC_ADDR} ]]; then
    collator_args+=(--public-addr ${PUBLIC_ADDR})
fi

if [[ ! -z ${STATE_PRUNING} ]]; then
    collator_args+=(--state-pruning ${STATE_PRUNING})
fi

if [[ ! -z ${BLOCKS_PRUNING} ]]; then
    collator_args+=(--blocks-pruning ${BLOCKS_PRUNING})
fi

if [[ ! -z ${LISTEN_ADDR} ]]; then
    collator_args+=(--listen-addr ${LISTEN_ADDR})
else
    collator_args+=(--port 40333)
fi

relay_args=(
    --execution wasm
    --base-path $2/relay-data/
    --chain $RESOURCES_PATH/${relay_chain}.json
    --port 30333
    --rpc-port 9944
    --rpc-external
    --rpc-cors all
    --wasm-execution Compiled
)

if [[ ! -z ${RELAY_PRUNING} ]]; then
    relay_args+=(--pruning ${RELAY_PRUNING})
else
    relay_args+=(--pruning 10000)
fi

if [[ ! -z ${RPC_MAX_CONNECTIONS} ]]; then
    collator_args+=(--rpc-max-connections ${RPC_MAX_CONNECTIONS})
    relay_args+=(--rpc-max-connections ${RPC_MAX_CONNECTIONS})
fi

if [[ ! -z ${RPC_MAX_REQUEST_SIZE} ]]; then
    collator_args+=(--rpc-max-request-size ${RPC_MAX_REQUEST_SIZE})
    relay_args+=(--rpc-max-request-size ${RPC_MAX_REQUEST_SIZE})
fi

if [[ ! -z ${RPC_MAX_RESPONSE_SIZE} ]]; then
    collator_args+=(--rpc-max-response-size ${RPC_MAX_RESPONSE_SIZE})
    relay_args+=(--rpc-max-response-size ${RPC_MAX_RESPONSE_SIZE})
fi

if [[ ! -z ${RPC_METHODS} ]]; then
    collator_args+=(--rpc-methods ${RPC_METHODS})
    relay_args+=(--rpc-methods ${RPC_METHODS})
fi


# collator_args+=($chain_spec)

#/target/release/hashed key insert --base-path ./collator-data $chain_spec --scheme sr25519 --suri "${MNEMO}" --key-type aura
# echo "${collator_args[@]}" -- "${relay_args[@]}"

$BASE_PATH/target/release/hashed-parachain "${collator_args[@]}" -- "${relay_args[@]}"
