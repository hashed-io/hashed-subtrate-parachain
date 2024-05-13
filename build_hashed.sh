#!/bin/bash


./scripts/build/assert_current_state.sh

 if [ $? -ne 0 ]; then
    echo "Current state assertion failed, exiting..."
    exit 1
fi

./scripts/build/setup_for_hashed.sh

cargo build "$@"

./scripts/build/setup_for_md5.sh

echo "Finished building hashed runtime"
