import * as wasm from "chip-eight";

// start application
let data = wasm.setup()

function change_rom(romName) {
    // clear previous inteval 
    d 

    // setup current rom
    wasm.setup_rom(data, romName)
    setInterval(data.run, data.interval())
}

// setup the callback for the rom drop down

