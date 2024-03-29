#!/usr/bin/env bash
cargo make all_release
gzip -f ./pkg/feh_sim_seed_bg.wasm
mv ./pkg/feh_sim_seed_bg.wasm.gz ./pkg/feh_sim_seed_bg.wasm
aws s3 cp . s3://fehsimseed-staging --recursive --exclude "*" --include "index.html" --include "pkg/feh_sim_seed.js" --include "style.css"
aws s3 cp ./pkg/feh_sim_seed_bg.wasm s3://fehsimseed-staging/pkg/feh_sim_seed_bg.wasm --content-type application/wasm --content-encoding gzip
