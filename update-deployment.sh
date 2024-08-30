#!/bin/bash

docker compose pull
docker compose down
docker volume rm wordplay_frontend-build
docker compose up --remove-orphans --detach
