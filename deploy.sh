#!/usr/bin/env bash
aws s3 sync s3://fehsimseed-staging s3://fehsimseed
aws cloudfront create-invalidation --distribution-id EC9ULX41LPJXL --paths "/*"