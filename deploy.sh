#!/usr/bin/env bash
/bin/sh build_release.sh
aws s3 cp . s3://fehsimseed --recursive --exclude "*" --include "*.html" --include "*.js" --include "pkg/package.js" --include "style.css" --profile "S3"
aws s3 cp ./pkg/package_bg.wasm s3://fehsimseed/pkg/package_bg.wasm --content-type application/wasm --profile S3