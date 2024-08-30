#!/bin/bash

docker buildx build --platform linux/amd64 --push -t ghcr.io/skearya/wordplay-frontend-builder client
docker buildx build --platform linux/amd64 --push -t ghcr.io/skearya/wordplay-server server
