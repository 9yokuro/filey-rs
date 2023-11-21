#!/bin/bash

# This script creates files and directories needed by the test suit.

set -eu

function check() {
    if [ ! -e "Cargo.toml" ]; then
        echo "Cargo.toml not found"
        return 1
    else
        return 0
    fi
}

function create() {
    export CURRENT_DIR=$(pwd)
    mkdir -p $CURRENT_DIR/test/a
    touch $CURRENT_DIR/test/test.txt
    ln -s $CURRENT_DIR/test/test.txt $CURRENT_DIR/test/test_symlink.txt
}

function main() {
    check
    if [ $? = 0 ]; then
        create
    fi
}

main
