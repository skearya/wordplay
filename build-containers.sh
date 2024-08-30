docker build -t ghcr.io/skearya/wordplay-frontend-builder:latest client
docker push ghcr.io/skearya/wordplay-frontend-builder:latest

docker build -t ghcr.io/skearya/wordplay-server:latest server
docker push ghcr.io/skearya/wordplay-server:latest
