#!/bin/bash

trunk build --public-url "/sardips" --release

echo "Done building trunk. Copying to frontend"

rm -rf ../../resume-site/public/sardips

mv ./dist ../../resume-site/public/sardips

echo "Build completed successfully."