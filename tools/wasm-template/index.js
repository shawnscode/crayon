import('./dist/native')
    .then(wasm => {
        wasm.wasm_main();
    })
    .catch(console.error);