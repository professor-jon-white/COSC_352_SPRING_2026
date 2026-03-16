#!/bin/bash
IMAGE_NAME="bpd-crime-dashboard"
PORT=3838

echo "Building BPD Dashboard Docker Image..."
docker build -t $IMAGE_NAME .

echo "Starting container on http://localhost:$PORT..."
#remove old container if exists
docker rm -f $IMAGE_NAME 2>/dev/null

#run container
docker run -d \
    --name $IMAGE_NAME \
    -p $PORT:3838 \
    $IMAGE_NAME

echo "-------------------------------------------------------"
echo "DONE! Access the dashboard at: http://localhost:$PORT"
echo "To stop the dashboard, run: docker stop $IMAGE_NAME"
echo "-------------------------------------------------------"