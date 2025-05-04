#!/bin/bash

trunk build

echo "Done building trunk. Copying to frontend"

old_dir=$(pwd)

rm -rf ../../thing-happend/frontend/public/sardips

mv ./dist ../../thing-happend/frontend/public/sardips

cd ../../thing-happend/frontend

pnpm run build

cd $old_dir

echo "Build completed successfully."