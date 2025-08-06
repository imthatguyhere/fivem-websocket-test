#!/bin/bash

#=-- Move into the project directory
cd test_websocket_server || { echo "Directory not found"; exit 1; }

#=-- Open index.html in the default browser
xdg-open index.html > /dev/null 2>&1 &
