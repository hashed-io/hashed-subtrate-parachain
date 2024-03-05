#!/usr/bin/env bash

# Builds docker image for full running network, relaychain plus hashed dev collator

usage="./build-full-network-image <tag>"
if [ $# -ne 1 ]; then
    echo $usage
    exit 1
fi

docker build --no-cache -f Dockerfile.fullnetwork -t sebastianmontero/hashed-full-network:$1 .
