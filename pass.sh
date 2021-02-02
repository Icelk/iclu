#!/bin/bash

length=$1
set="ascii"

help="\
Usage: pass.sh PASSWORD_LENGTH [CHARACTER_SET]

If no arguments are supplied, will generate a strong password with a length of 20.

Generates a random password with PASSWORD_LENGTH.
A custom CHARACTER_SET can be used. See \`ran --help\` for a complete list.
By default, all Ascii characters are in the range. If less characters are desired (e.g. if the website does not support all) you can use the \`password\` set.
Default length is 20 because it generates more than 128-bits of entropy (95^20 / 2^128 > 1, 95 is the character count in the default set).
"

if [[ "$1" == "--help" ]]; then
    echo -e "$help"
    exit 1
fi

if [ -z "$length" ]; then
    length="20"
fi
if [ -n "$2" ]; then
    set="$2"
fi

ran -n $length $set | byc
