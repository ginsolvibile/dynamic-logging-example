#!/bin/bash

docker run -it --rm -v "$(pwd):/mount:ro" \
    ghcr.io/ktr0731/evans:latest \
    --path ./ \
    --proto proto/dynlog.proto \
    --host host.docker.internal \
    --port 50051 \
    repl
