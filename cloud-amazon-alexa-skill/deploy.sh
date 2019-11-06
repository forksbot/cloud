#!/bin/bash
CRATE_NAME=cloud-amazon-alexa-skill
TOOLCHAIN_DIR=x86_64-unknown-linux-musl/

cargo build -p $CRATE_NAME --release
cp ./target/${TOOLCHAIN_DIR}release/$CRATE_NAME ./bootstrap && strip bootstrap && zip lambda.zip bootstrap && rm bootstrap