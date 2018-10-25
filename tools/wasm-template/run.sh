BASEDIR=$(dirname "$0")

# cargo build --target wasm32-unknown-unknown
# wasm-bindgen target/wasm32-unknown-unknown/debug/crayon.wasm --out-dir $BASEDIR/dist

cargo build --example $1 --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/examples/$1.wasm $BASEDIR/dist/intermediate/native.wasm
wasm-bindgen $BASEDIR/dist/intermediate/native.wasm --out-dir $BASEDIR/dist

cd $BASEDIR

# npm install
npm run serve