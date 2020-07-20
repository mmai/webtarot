#!/usr/bin/env sh

REPO=webtarot

# Remove old containers & image
IMAGE=$(docker images --format '{{.Repository}}:{{.Tag}}' | grep $REPO )
IMAGEID=$(docker images --format '{{.Repository}}:{{.ID}}' | grep $REPO | awk -F':' '{print $2}')

# echo $IMAGE

docker ps -a | grep $IMAGE | awk '{print $1 }' | xargs docker rm
docker rmi $IMAGE

# create new
nix-build webtarot_docker.nix
docker load < result

## ssh in container
# REPOTAG=$(docker images --format '{{.Repository}}:{{.Tag}}' | grep $REPO) 
# echo $REPOTAG
# docker run -it $REPOTAG /bin/bash
