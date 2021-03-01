use crate::{
    definitions::{cpu, display},
    opcode::{ChipOpcodes, Opcode, OpcodeTrait, Operation, ProgramCounter, ProgramCounterStep},
    timer::{TimedWorker, TimerCallback},
};

use super::InternalChipSet;

impl<W, S> ChipOpcodes for InternalChipSet<W, S>
where
    W: TimedWorker,
    S: TimerCallback,
{
    fn zero(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        log::debug!("opcode {:#X}", opcode);
        match opcode {
            0x00E0 => {
                // 00E0
                // clear display
                for row in self.display.iter_mut() {
                    for coloumn in row.iter_mut() {
                        *coloumn = false;
                    }
                }
                Ok((ProgramCounterStep::Next, Operation::Draw))
            }
            0x00EE => {
                // 00EE
                // Return from sub routine => pop from stack
                let pc = self.pop_stack()?;
                log::debug!("pc {:#X}", pc);
                Ok((ProgramCounterStep::Jump(pc), Operation::None))
            }
            _ => Err(format!(
                "An unsupported opcode was used {:#06X?}",
                self.opcode
            )),
        }
    }

    fn one(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 1NNN
        // Jumps to address NNN.
        Ok(ProgramCounterStep::Jump(opcode.nnn()))
    }

    fn two(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 2NNN
        // Calls subroutine at NNN 
        // and set's the program counter to the next opcode after the given stack push

        match self.push_stack(self.program_counter + ProgramCounterStep::Next.step()) {
            Ok(_) => Ok(ProgramCounterStep::Jump(opcode.nnn())),
            Err(err) => Err(err.to_string()),
        }
    }

    fn three(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 3XNN
        // Skips the next instruction if VX equals NN. (Usually the next instruction is a jump to
        // skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] == nn))
    }

    fn four(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 4XNN
        // Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
        // jump to skip a code block)
        let (x, nn) = opcode.xnn();
        Ok(ProgramCounterStep::cond(self.registers[x] != nn))
    }

    fn five(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 5XY0
        // Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
        // skip a code block)
        match opcode.xyn() {
            (x, y, 0) => Ok(ProgramCounterStep::cond(
                self.registers[x] == self.registers[y],
            )),
            _ => Err(format!("An unsupported opcode was used {:#06X?}", opcode)),
        }
    }

    fn six(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 6XNN
        // Sets VX to NN.
        let (x, nn) = opcode.xnn();
        self.registers[x] = nn;
        Ok(ProgramCounterStep::Next)
    }

    fn seven(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 7XNN
        // Adds NN to VX. (Carry flag is not changed)
        let (x, nn) = opcode.xnn();
        // let VX overflow, but ignore carry
        let res = self.registers[x].wrapping_add(nn);
        self.registers[x] = res;
        Ok(ProgramCounterStep::Next)
    }

    fn eight(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // remove the middle 8 bits for calculations
        let (x, y, n) = opcode.xyn();
        match n {
            0x0 => {
                // 8XY0
                // Sets VX to the value of VY.
                self.registers[x] = self.registers[y];
            }
            0x1 => {
                // 8XY1
                // Sets VX to VX or VY. (Bitwise OR operation)
                self.registers[x] = self.registers[x] | self.registers[y];
            }
            0x2 => {
                // 8XY2
                // Sets VX to VX and VY. (Bitwise AND operation)
                self.registers[x] = self.registers[x] & self.registers[y];
            }
            0x3 => {
                // 8XY3
                // Sets VX to VX xor VY.
                self.registers[x] = self.registers[x] ^ self.registers[y];
            }
            0x4 => {
                // 8XY4
                // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                let left = self.registers[x] as u16;
                let right = self.registers[y] as u16;
                let res = left + right;
                let carry = res & 0x0100 == 0x0100;
                self.registers[x] = res as u8;
                self.registers[cpu::register::LAST] = if carry { 1 } else { 0 };
            }
            0x5 => {
                // 8XY5
                // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let left = self.registers[x] as u16;
                let right = ((!self.registers[y]).wrapping_add(1)) as u16;
                let res = left + right;
                let carry = (res & 0x0100) == 0x0100;
                self.registers[x] = res as u8;
                self.registers[cpu::register::LAST] = if carry { 1 } else { 0 };
            }
            0x6 => {
                // 8XY6
                // Stores the least significant bit of VX in VF and then shifts VX to the right
                // by 1.
                self.registers[cpu::register::LAST] = self.registers[x] & 1;
                self.registers[x] = self.registers[x] >> 1;
            }
            0x7 => {
                // 8XY7
                // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there
                // isn't.
                let left = self.registers[y] as u16;
                let right = ((!self.registers[x]).wrapping_add(1)) as u16;
                let res = left + right;
                let carry = (res & 0x0100) == 0x0100;
                self.registers[x] = res as u8;
                self.registers[cpu::register::LAST] = if carry { 1 } else { 0 };
            }
            0xE => {
                // 8XYE
                // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
                const SHIFT_SIGNIFICANT: u8 = 7;
                const AND_SIGNIFICANT: u8 = 1 << SHIFT_SIGNIFICANT;
                self.registers[cpu::register::LAST] =
                    (self.registers[x] & AND_SIGNIFICANT) >> SHIFT_SIGNIFICANT;
                self.registers[x] = self.registers[x] << 1;
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        }
        // increment the program counter by one
        Ok(ProgramCounterStep::Next)
    }

    fn nine(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // 9XY0
        // Skips the next instruction if VX doesn't equal VY. (Usually the next instruction is
        // a jump to skip a code block)
        match opcode.xyn() {
            (x, y, 0) => Ok(ProgramCounterStep::cond(
                self.registers[x] != self.registers[y],
            )),
            _ => Err(format!("An unsupported opcode was used {:#06X?}", opcode)),
        }
    }

    fn a(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // ANNN
        // Sets I to the address NNN.
        self.index_register = opcode.nnn() as usize;
        Ok(ProgramCounterStep::Next)
    }

    fn b(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // BNNN
        // Jumps to the address NNN plus V0.
        let nnn = opcode.nnn();
        let v0 = self.registers[0] as usize;
        Ok(ProgramCounterStep::Jump(v0 + nnn))
    }

    fn c(&mut self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        // CXNN
        // Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
        // and NN.

        let (x, nn) = opcode.xnn();
        // using a fill bytes call here, as the trait RngCore does not
        // support random u8.
        let mut rand: [u8; 1] = [0];
        self.rng.fill_bytes(&mut rand);
        self.registers[x] = nn & rand[0];
        Ok(ProgramCounterStep::Next)
    }

    fn d(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        // DXYN
        // Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N
        // pixels. Each row of 8 pixels is read as bit-coded starting from memory location I; I
        // value doesn’t change after the execution of this instruction. As described above, VF is
        // set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and
        // to 0 if that doesn’t happen
        // see https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

        let (reg_x, reg_y, n) = opcode.xyn();

        let index = self.index_register;
        let coorx = self.registers[reg_x] as usize;
        let coory = self.registers[reg_y] as usize;

        let coorx = coorx % display::HEIGHT;
        let coory = coory % display::WIDTH;

        // Set VF to 0
        self.registers[cpu::register::LAST] = 0;

        const BYTE: usize = 8;

        // Get one byte of sprite data from the memory address in the I register
        for (i, row) in self.memory[index..(index + n)].iter().enumerate() {
            let y = coory + i;

            if y >= display::WIDTH {
                break;
            }

            // - If the current pixel in the sprite row is 'on' and the pixel at coordinates X,Y
            //   on the screen is also 'on', turn 'off' the pixel and set VF to '1'.
            // - Or if the current pixel in the sprite row is 'on' and the screen pixel is 'not',
            //  draw the pixel at the X and Y coordinates.

            // Attention about the endianess of the system.

            for (m, j) in (0..BYTE).rev().zip(0..BYTE) {
                let mask = 1 << m;
                let x = coorx + j;

                if x >= display::HEIGHT {
                    break;
                }

                let cpixel = (*row & mask) == mask;

                if !cpixel {
                    continue;
                }

                let spixel = self.display[y][x];

                self.display[y][x] = !spixel;

                if spixel {
                    self.registers[cpu::register::LAST] = 1;
                }
            }
        }

        Ok((ProgramCounterStep::Next, Operation::Draw))
    }

    fn e(&self, opcode: Opcode) -> Result<ProgramCounterStep, String> {
        let (x, nn) = opcode.xnn();
        let step = match nn {
            0x9E => {
                // EX9E
                // Skips the next instruction if the key stored in VX is pressed. (Usually the next
                // instruction is a jump to skip a code block)
                ProgramCounterStep::cond(self.keyboard.get_keys()[self.registers[x] as usize])
            }
            0xA1 => {
                // EXA1
                // Skips the next instruction if the key stored in VX isn't pressed. (Usually the
                // next instruction is a jump to skip a code block)
                ProgramCounterStep::cond(!self.keyboard.get_keys()[self.registers[x] as usize])
            }
            _ => {
                // directly return with the given error
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ));
            }
        };
        Ok(step)
    }

    fn f(&mut self, opcode: Opcode) -> Result<(ProgramCounterStep, Operation), String> {
        let (x, nn) = opcode.xnn();
        let mut op = Operation::None;
        let mut pcs = ProgramCounterStep::Next;
        match nn {
            0x07 => {
                // FX07
                // Sets VX to the value of the delay timer.
                self.registers[x] = self.get_delay_timer();
            }
            0x0A => {
                // FX0A
                // A key press is awaited, and then stored in VX. (Blocking Operation. All
                // instruction halted until next key event)
                let callback_after_keypress = move |chip: &mut Self| {
                    let last = chip.keyboard.get_last().expect(
                        "The contract that states a last key has to be set was not fullfilled.",
                    );
                    chip.registers[x] = last.get_index() as u8;
                    // move the counter to the next instruction
                    chip.step(ProgramCounterStep::Next);
                };

                op = Operation::Wait;
                // don't change the counter until the rest of the function is called.
                pcs = ProgramCounterStep::None;

                self.preprocessor = Some(Box::new(callback_after_keypress));
            }
            0x15 => {
                // FX15
                // Sets the delay timer to VX.
                self.delay_timer.set_value(self.registers[x]);
            }
            0x18 => {
                // FX18
                // Sets the sound timer to VX.
                self.sound_timer.set_value(self.registers[x]);
            }
            0x1E => {
                // FX1E
                // Adds VX to I. VF is set to 1 when there is a range overflow (I+VX>0xFFF), and to
                // 0 when there isn't. (not used in this system)
                //
                // Adds VX to I. VF is not affected.[c]
                let xi = self.registers[x] as usize;
                self.index_register = self.index_register.wrapping_add(xi);
            }
            0x29 => {
                // FX29
                // Sets I to the location of the sprite for the character in VX. Characters 0-F (in
                // hexadecimal) are represented by a 4x5 font.
                // TODO: implement sprite offset
                let val = self.registers[x] as usize;
                assert!(
                    val <= 0xF,
                    "There was a too large number in register <{:#X}> for hex representation.",
                    x
                );
                self.index_register = display::fontset::LOCATION + 5 * val;
            }
            0x33 => {
                // FX33
                // Stores the binary-coded decimal representation of VX, with the most significant
                // of three digits at the address in I, the middle digit at I plus 1, and the least
                // significant digit at I plus 2. (In other words, take the decimal representation
                // of VX, place the hundreds digit in memory at location in I, the tens digit at
                // location I+1, and the ones digit at location I+2.)
                let i = self.index_register;
                let r = self.registers[x];

                self.memory[i] = r / 100; // 246u8 / 100 => 2
                self.memory[i + 1] = r / 10 % 10; // 246u8 / 10 => 24 % 10 => 4
                self.memory[i + 2] = r % 10; // 246u8 % 10 => 6
            }
            0x55 => {
                // FX55
                // Stores V0 to VX (including VX) in memory starting at address I. The offset from I
                // is increased by 1 for each value written, but I itself is left unmodified.
                let index = self.index_register;
                self.memory[index..=(index + x)].copy_from_slice(&self.registers[..=x]);
            }
            0x65 => {
                // FX65
                // Fills V0 to VX (including VX) with values from memory starting at address I. The
                // offset from I is increased by 1 for each value written, but I itself is left
                // unmodified.
                let index = self.index_register;
                self.registers[..=x].copy_from_slice(&self.memory[index..=(index + x)]);
            }
            _ => {
                return Err(format!(
                    "An unsupported opcode was used {:#06X?}",
                    self.opcode
                ))
            }
        }
        Ok((pcs, op))
    }
}
