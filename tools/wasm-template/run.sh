BASEDIR=$(dirname "$0")

set -e
cargo build --example $1 --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/examples/$1.wasm $BASEDIR/dist/intermediate/native.wasm
wasm-bindgen $BASEDIR/dist/intermediate/native.wasm --out-dir $BASEDIR/dist

cd $BASEDIR
npm run serve