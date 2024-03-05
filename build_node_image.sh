#!/usr/bin/env bash


# Builds docker image for a hashed or md5 collator node.

usage="./build_node_image <hashed|md5> <tag>"
if [ $# -ne 2 ]; then
    echo $usage
    exit 1
fi

if [[ $1 != 'hashed' && $1 != 'md5' ]]; then
    echo $usage
    exit 1
fi

docker build -f Dockerfile.$1 -t sebastianmontero/hashed-substrate-collator-$1:$2 .
