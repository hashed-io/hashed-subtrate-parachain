#!/bin/bash

# Check if the correct number of arguments is provided
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <file_to_check> <strings_file>"
    exit 1
fi

file_to_check=$1
strings_file=$2

if [ ! -e "$file_to_check" ]; then
    echo "File to check does not exist: $file_to_check"
    exit 1
fi

if [ ! -e "$strings_file" ]; then
    echo "Strings file does not exist: $strings_file"
    exit 1
fi

IFS=$'\n' read -d '' -r -a strings_to_check < "$strings_file"


# Function to check if a string exists in the file
check_string() {
    echo "checking string: $1 is in file $file_to_check"
    grep -qF "$1" "$file_to_check"
}

# Iterate through the strings and check each one
for string_to_check in "${strings_to_check[@]}"; do
    check_string "$string_to_check"
    if [ $? -ne 0 ]; then
        echo "Error: String: $string_to_check not found in the file: $file_to_check"
        exit 1
    fi
done

# If all strings are found, print success message
echo "All strings found in file: $file_to_check"
exit 0
