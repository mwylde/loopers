#!/bin/bash
docker build . -t rust_cross_compile/aarch64 -f Dockerfile.aarch64
docker run --rm -ti -v `pwd`:/app rust_cross_compile/aarch64