#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 input.csv"
    exit 1
fi

input_file="$1"

awk -F',' 'NR > 1 {pair = $1","$2; if (!seen[pair]++) print pair}' "${1}" | paste -sd' ' -
