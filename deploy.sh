#!/bin/bash

~/.cargo/bin/wasm-pack build || exit
cd www || exit

rm -rf dist

npm run build || exit

cd dist || exit
zip -r ../ld47 ./*
cd ..

scp ld47.zip necauqua.dev:.

rm ld47.zip

ssh necauqua.dev 'bash -c "rm -rf ld47; unzip ld47.zip -d ld47; rm ld47.zip"'

