import * as wasm from "chip-eight";

// start application
let data = wasm.init()

function change_rom(romName) {
    data.start(romName)
}

// change_rom("TESTOPCODE")
change_rom("TANK")
// change_rom("IBMLOGO")
// setup the callback for the rom drop down
