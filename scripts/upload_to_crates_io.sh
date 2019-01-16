#!/bin/bash -ex

cargo login "$CRATES_TOKEN"

cargo publish
