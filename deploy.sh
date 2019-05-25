#!/usr/bin/env bash
cargo make all_release
aws s3 cp . s3://fehsimseed/fehstatsim --recursive --exclude "*" --include "*.html" --include "*.js" --include "pkg/package.js" --include "style.css" --profile S3
aws s3 cp ./pkg/package_bg.wasm s3://fehsimseed/fehstatsim/pkg/package_bg.wasm --content-type application/wasm --profile S3