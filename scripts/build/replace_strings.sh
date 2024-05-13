#!/bin/bash

# Check if the correct number of arguments is provided
if [ "$#" -lt 2 ]; then
    echo "Usage: $0 <file_path> <find_strings_file> <replace_strings_file>"
    exit 1
fi

file_path=$1
find_strings_file=$2
replace_strings_file=$3

if [ ! -e "$file_path" ]; then
    echo "File to check does not exist: $file_path"
    exit 1
fi

if [ ! -e "$find_strings_file" ]; then
    echo "Find strings file does not exist: $find_strings_file"
    exit 1
fi

if [ ! -e "$replace_strings_file" ]; then
    echo "Replace strings file does not exist: $replace_strings_file"
    exit 1
fi

IFS=$'\n' read -d '' -r -a find_strings < "$find_strings_file"

IFS=$'\n' read -d '' -r -a replace_strings < "$replace_strings_file"

if [ "${#find_strings[@]}" -ne "${#replace_strings[@]}" ]; then
    echo "Number of strings to find should be the same to the number of strings to replace"
    exit 1
fi


# Function to check if a string exists in the file
check_string() {
    echo "checking string: $1 is in file $file_path"
    grep -qF "$1" "$file_path"
}

# Iterate through the strings and check each one
for ((i=0; i<${#find_strings[@]}; i++)); do
    sed -i "s/${find_strings[$i]}/${replace_strings[$i]}/g" "$file_path"
done



# If all strings are found, print success message
echo "Strings have been replaced in file: $file_path"
exit 0
