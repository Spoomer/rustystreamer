#! bin/bash
# install cross via cargo: cargo install cross
# target = raspberry pi 2-4
cross build -r --target armv7-unknown-linux-gnueabihf
mkdir release
cp -r ./assets/ ./release/assets/
cp -r ./views/ ./release/views/
cp ./target/armv7-unknown-linux-gnueabihf/release/rustystreamer ./release/rustystreamer
cp ./config.json ./release/config.json.template
zip -rm release.zip release
