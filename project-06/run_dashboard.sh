#!/bin/bash

docker build -t project6-dashboard .

docker run -p 3838:3838 project6-dashboard

echo "Dashboard running at http://localhost:3838"
