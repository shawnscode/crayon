import('./dist/native')
    .then(wasm => {
        wasm.run();
    })
    .catch(console.error);