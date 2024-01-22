#!/bin/bash

./scripts/build/assert_contains_strings.sh "./runtime/src/lib.rs" "./scripts/build/runtime_lib_md5_strings.txt"
./scripts/build/assert_contains_strings.sh "./runtime/src/xcm_config.rs" "./scripts/build/runtime_xcm_config_md5_strings.txt"
