#!/bin/bash


./scripts/build/assert_current_state.sh

 if [ $? -ne 0 ]; then
    echo "Current state assertion failed, exiting..."
    exit 1
fi

cargo build "$@"

echo "Finished building md5 runtime"
