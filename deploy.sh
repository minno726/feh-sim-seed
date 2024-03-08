#!/usr/bin/env bash
aws s3 sync s3://fehsimseed-staging s3://fehstatsim-v1.fullyconcentrated.net
aws cloudfront create-invalidation --distribution-id E38MT0DWC5VPU8 --paths "/*"