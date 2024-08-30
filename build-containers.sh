docker buildx build --platform linux/amd64 --push -t ghcr.io/skearya/wordplay-frontend-builder:latest client
docker buildx build --platform linux/amd64 --push -t ghcr.io/skearya/wordplay-server:latest server
