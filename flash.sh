#! /bin/bash

set -e

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "usage: $0 <path-to-binary.elf>" >&2
    exit 1
fi

if [ "$#" -lt 1 ]; then
    echo "$0: Expecting a .elf file" >&2
    exit 1
fi

FILE=$1
shift

cargo build $@
avrdude -patmega328p -carduino -P/dev/ttyACM0 -D -U flash:w:"$FILE":e
