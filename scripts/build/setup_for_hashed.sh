#!/bin/bash

./scripts/build/replace_strings.sh "./runtime/src/lib.rs" "./scripts/build/runtime_lib_md5_strings.txt" "./scripts/build/runtime_lib_hashed_strings.txt"
./scripts/build/replace_strings.sh "./runtime/src/xcm_config.rs" "./scripts/build/runtime_xcm_config_md5_strings.txt" "./scripts/build/runtime_xcm_config_hashed_strings.txt"
