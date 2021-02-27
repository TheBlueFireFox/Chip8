import * as wasm from "chip-eight";

// start application
let data = wasm.setup()

function change_rom(romName) {
    data.start(romName)
}

change_rom("TESTOPCODE")
// setup the callback for the rom drop down
