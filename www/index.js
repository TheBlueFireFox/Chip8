import * as wasm from "chip-eight";

// start application
let data = wasm.setup()
setInterval(data.run, data.interval())
