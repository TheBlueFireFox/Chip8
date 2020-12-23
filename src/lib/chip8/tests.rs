use {
    super::ChipSet,
    crate::{
        definitions::{
            OPCODE_BYTE_SIZE, PROGRAM_COUNTER, REGISTER_LAST, REGISTER_SIZE, STACK_NESTING,
        },
        opcode::{ChipOpcodes, Opcode, Operation, ProgramCounter, ProgramCounterStep},
        resources::{Rom, RomArchives},
    },
    lazy_static::lazy_static,
    std::panic,
};

const ROM_NAME: &'static str = "15PUZZLE";

lazy_static! {
    /// preloading this as it get's called multiple times per unit
    static ref BASE_ROM : Rom = {
        let mut ra = RomArchives::new();
        // unwrap is safe here as this never even should be able to crash
        // and in the unlikely case that it does a panic is correct.
        ra.get_file_data(ROM_NAME).unwrap()
    };
}

pub(super) fn get_base() -> Rom {
    BASE_ROM.clone()
}

/// will setup the default configured chip
pub(super) fn get_default_chip() -> ChipSet {
    let rom = get_base();
    setup_chip(rom)
}

pub(super) fn setup_chip(rom: Rom) -> ChipSet {
    let mut chip = ChipSet::new(rom);
    // fill up register with random values
    assert_eq!(chip.registers.len(), 16);
    chip.registers = (0..REGISTER_SIZE).map(|_| rand::random()).collect();

    assert_eq!(chip.registers.len(), 16);
    chip
}

/// Will write the opcode to the memory location specified
pub(super) fn write_opcode_to_memory(memory: &mut [u8], from: usize, opcode: Opcode) {
    write_slice_to_memory(memory, from, &opcode.to_be_bytes());
}

/// Will write the slice to the memory location specified
pub(super) fn write_slice_to_memory(memory: &mut [u8], from: usize, data: &[u8]) {
    for i in 0..data.len() {
        memory[from + i] = data[i];
    }
}

#[test]
/// test reading of the first opcode
fn test_set_opcode() {
    let mut chip = get_default_chip();
    let opcode = 0xA00A;
    write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

    assert!(chip.set_opcode().is_ok());

    assert_eq!(chip.opcode, opcode);
}

#[test]
/// testing internal functionality of popping and pushing into the stack
fn test_push_pop_stack() {
    let mut chip = get_default_chip();

    // check empty initial stack
    assert_eq!(0, chip.stack_pointer);

    let next_counter = 0x0133 + PROGRAM_COUNTER;

    for i in 0..STACK_NESTING {
        // as the stack is empty just accept the result
        assert_eq!(Ok(()), chip.push_stack(next_counter + i * 8));
    }
    // check for the correct error message
    assert_eq!(Err("Stack is full!"), chip.push_stack(next_counter));

    // check if the stack counter moved as expected
    assert_eq!(STACK_NESTING, chip.stack_pointer);
    // pop the stack
    for i in (0..STACK_NESTING).rev() {
        assert_eq!(Ok(next_counter + i * 8), chip.pop_stack());
    }
    assert_eq!(0, chip.stack_pointer);
    // test if stack is now empty
    assert_eq!(Err("Stack is empty!"), chip.pop_stack());
}

#[test]
fn test_step() {
    let mut chip = get_default_chip();
    let mut pc = chip.program_counter;

    pc += OPCODE_BYTE_SIZE;
    chip.step(ProgramCounterStep::Next);
    assert_eq!(chip.program_counter, pc);

    pc += 2 * OPCODE_BYTE_SIZE;
    chip.step(ProgramCounterStep::Skip);
    assert_eq!(chip.program_counter, pc);

    pc += 8 * OPCODE_BYTE_SIZE;
    chip.step(ProgramCounterStep::Jump(pc));
    assert_eq!(chip.program_counter, pc);

    chip.step(ProgramCounterStep::None);
    assert_eq!(chip.program_counter, pc);
}

#[test]
#[should_panic(expected = "Memory out of bounds error!")]
fn test_step_panic_lower_bound() {
    let mut chip = get_default_chip();
    let pc = PROGRAM_COUNTER - 1;
    chip.step(ProgramCounterStep::Jump(pc));
}

#[test]
#[should_panic(expected = "Memory out of bounds error!")]
fn test_step_panic_upper_bound() {
    let mut chip = get_default_chip();
    let pc = chip.memory.len();
    chip.step(ProgramCounterStep::Jump(pc));
}

mod zero {
    use super::*;

    #[test]
    /// test clear display opcode and next (for coverage)
    /// `0x00E0`
    fn test_clear_display_opcode() {

        let mut chip = get_default_chip();

        let curr_pc = chip.program_counter;

        // as the first opcode used is already clear screen no
        // modifications are needed.

        // run - if there was no panic it worked as intended
        assert_eq!(chip.next(), Ok(Operation::Clear));

        assert_eq!(curr_pc + OPCODE_BYTE_SIZE, chip.program_counter);
    }

    #[test]
    /// test return from subroutine
    /// `0x00EE`
    fn test_return_subrutine() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;
        // set up test
        let base = 0x234;
        let opcode: Opcode = 0x2000 ^ base;

        // write the to subroutine to memory
        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));
        // set opcode
        let opcode = 0x00EE;

        // write bytes to chip memory
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);
        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.next());

        assert_eq!(curr_pc, chip.program_counter)
    }

    #[test]
    fn test_illigal_zero_opcode() {
        let mut chip = get_default_chip();
        let opcode = 0x00EA;
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);
        assert_eq!(
            Err("An unsupported opcode was used 0x00EA".to_string()),
            chip.next()
        );
    }
}
mod one {
    use super::*;

    #[test]
    /// test a simple jump to the next address
    /// `1NNN`
    fn test_jump_address() {
        let mut chip = get_default_chip();
        let base = 0x0234;
        let opcode = 0x1000 ^ base as Opcode;
        // let _ = chip.move_program_counter(base);
        chip.step(ProgramCounterStep::Jump(base));
        chip.opcode = opcode;

        assert_eq!(chip.calc(opcode), Ok(Operation::None));

        assert_eq!(base, chip.program_counter);
    }
}

mod two {
    use super::*;
    #[test]
    /// test inserting a location into the stack
    /// "2NNN"
    fn test_call_subrutine() {
        let mut chip = get_default_chip();
        let base = 0x234;
        let opcode = 0x2000 ^ base;
        let curr_pc = chip.program_counter;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(base as usize, chip.program_counter);

        assert_eq!(curr_pc, chip.stack[0]);
    }
}
mod three {
    use super::*;

    #[test]
    /// test the skip instruction if equal method
    /// `3XNN`
    fn test_skip_instruction_if_const_equals() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let solution = 0x3;
        // skip register 1 if it is equal to 03
        let opcode = 0x3 << (3 * 4) ^ (register << (2 * 4)) ^ solution;

        let curr_pc = chip.program_counter;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = solution as u8;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }
}
mod four {
    use super::*;
    #[test]
    /// `4XNN`
    /// Skips the next instruction if VX doesn't equal NN. (Usually the next instruction is a
    /// jump to skip a code block)
    fn test_skip_instruction_if_const_not_equals() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let solution = 0x3;
        // skip register 1 if it is not equal to 03
        let opcode = 0x4 << (3 * 4) ^ (register << (2 * 4)) ^ solution;

        // will not skip next instruction
        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = solution as u8;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        // skip next block because it's not equal
        let curr_pc = chip.program_counter;
        chip.registers[register as usize] = 0x66;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }
}
mod five {
    use super::*;

    #[test]
    /// 5XY0
    /// Skips the next instruction if VX equals VY. (Usually the next instruction is a jump to
    /// skip a code block)
    fn test_skip_instruction_if_register_equals() {
        let mut chip = get_default_chip();
        let registery = 0x1;
        let registerx = 0x2;
        // skip register 1 if VY is not equals to VX
        let opcode = 0x5 << (3 * 4) ^ (registerx << (2 * 4)) ^ (registery << (1 * 4) ^ 0);

        // setup register for a none skip
        chip.registers[registerx as usize] = 0x6;
        chip.registers[registery as usize] = 0x66;
        // will not skip next instruction
        let curr_pc = chip.program_counter;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);

        // skip next block because it's not equal
        // setup register
        chip.registers[registerx as usize] = 0x66;
        chip.registers[registery as usize] = 0x66;
        // copy current state of program counter
        let curr_pc = chip.program_counter;
        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.program_counter, curr_pc + 2 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// mainly for coverage, but still simple to test
    fn test_five_false_opcode() {
        let mut chip = get_default_chip();
        let registery = 0x1;
        let registerx = 0x2;
        let pc = chip.program_counter;
        for i in 1..16 {
            let opcode = 0x5 << (3 * 4) ^ (registerx << (2 * 4)) ^ (registery << (1 * 4) ^ i);

            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            assert_eq!(
                chip.next(),
                Err(format!("An unsupported opcode was used {:#06X?}", opcode))
            );
            // assert that there were no movement
            assert_eq!(pc, chip.program_counter);
        }
    }
}
mod six {
    use super::*;

    #[test]
    /// 6XNN
    /// Sets VX to NN.
    fn test_set_vx_to_nn() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let value = 0x66 & chip.registers[register];
        let curr_pc = chip.program_counter;
        chip.registers[register] = value;
        // skip register 1 if VY is not equals to VX
        let opcode: Opcode = 0x6 << (3 * 4) ^ ((register as u16) << (2 * 4)) ^ (value as u16);

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(value, chip.registers[register]);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }
}

mod seven {
    use super::*;

    #[test]
    /// 7XNN
    /// Adds NN to VX. (Carry flag is not changed)
    fn test_add_nn_to_vx() {
        let mut chip = get_default_chip();
        let register = 0x1;
        let value: u8 = 0x66;
        let value_reg: u8 = 0xFA;
        let curr_pc = chip.program_counter;
        chip.registers[register] = value_reg;
        // skip register 1 if VY is not equals to VX
        let opcode: Opcode = 0x7 << (3 * 4) ^ ((register as u16) << (2 * 4)) ^ (value as u16);

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        let res = 0x60;
        assert_eq!(res, chip.registers[register]);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }
}
mod eight {
    use super::*;

    #[test]
    /// 8XY0
    /// Sets VX to the value of VY.
    fn test_move_value() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x0;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_ne!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_x], val_reg_y);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY1
    // Sets VX to VX or VY. (Bitwise OR operation)
    fn test_bitwise_or() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x1;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0xFE);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY1
    // Sets VX to VX or VY. (Bitwise OR operation)
    fn test_bitwise_and() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x2;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0x10);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY3
    // Sets VX to VX xor VY.
    fn test_bitwise_xor() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x3;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0xEE);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY4
    // Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
    fn test_addition_with_carry() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x4;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0x0E);
        assert_eq!(chip.registers[REGISTER_LAST], 1);
        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY5
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
    // isn't.
    fn test_substraction_with_carry() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0x14;
        let val_reg_y = 0xFA;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x5;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0x1A);
        assert_eq!(chip.registers[REGISTER_LAST], 1);
        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY5
    // VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there
    // isn't.
    fn test_least_sig_bit_and_shift_right() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0x9;

        let val_reg_x = 0x11;

        chip.registers[reg_x] = val_reg_x;

        assert_eq!(chip.registers[reg_x], val_reg_x);

        let command = 0x6;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0x08);
        assert_eq!(chip.registers[REGISTER_LAST], 1);
        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XY7
    // Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there
    // isn't.
    fn test_reverse_substraction_with_carry() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xF;

        let val_reg_x = 0xFA;
        let val_reg_y = 0x14;
        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        assert_eq!(chip.registers[reg_x], val_reg_x);
        assert_eq!(chip.registers[reg_y], val_reg_y);

        let command = 0x7;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0x1A);
        assert_eq!(chip.registers[REGISTER_LAST], 1);
        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    // 8XYE
    // Stores the most significant bit of VX in VF and then shifts VX to the left by 1.
    fn test_most_sig_bit_and_shift_left() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0x9;

        let val_reg_x = 0xF1;

        chip.registers[reg_x] = val_reg_x;

        assert_eq!(chip.registers[reg_x], val_reg_x);

        let command = 0xE;

        let opcode: Opcode =
            0x8 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ command;

        chip.opcode = opcode;

        assert_eq!(Ok(Operation::None), chip.calc(opcode));

        assert_eq!(chip.registers[reg_x], 0xE2);
        assert_eq!(chip.registers[REGISTER_LAST], 1);
        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }

    #[test]
    /// This test is mainly for correct coverage.
    fn test_eight_wrong_opcode() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let opcode: Opcode = 0x800A;
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

        assert_eq!(
            chip.next(),
            Err(format!("An unsupported opcode was used {:#06X?}", opcode))
        );

        assert_eq!(chip.program_counter, curr_pc);
    }
}
mod nine {
    use super::*;

    #[test]
    /// This test is mainly for correct coverage.
    fn test_nine_wrong_opcode() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xA;

        let val_reg_x = 0x1;
        let val_reg_y = 0xA;

        chip.registers[reg_x] = val_reg_x;
        chip.registers[reg_y] = val_reg_y;

        for i in 1..16 {
            let opcode: Opcode =
                0x9 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ i;
            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            assert_eq!(
                chip.next(),
                Err(format!("An unsupported opcode was used {:#06X?}", opcode))
            );

            assert_eq!(chip.program_counter, curr_pc);
        }
    }

    #[test]
    /// This test is mainly for correct coverage.
    fn test_skip_if_reg_not_equals() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let reg_x = 0x1;
        let reg_y = 0xA;

        let val_reg_x = 0x1;
        let val_reg_y = 0x1;

        let save = |reg: &mut [u8], (reg_x, val_x), (reg_y, val_y)| {
            reg[reg_x] = val_x;
            reg[reg_y] = val_y;
        };

        save(&mut chip.registers, (reg_x, val_reg_x), (reg_y, val_reg_y));

        let opcode: Opcode =
            0x9 << (3 * 4) ^ (reg_x as u16) << (2 * 4) ^ (reg_y as u16) << (1 * 4) ^ 0;
        {
            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            assert_eq!(chip.next(), Ok(Operation::None));

            assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
        }
        {
            let val_reg_y = 0x2;

            save(&mut chip.registers, (reg_x, val_reg_x), (reg_y, val_reg_y));

            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            assert_eq!(chip.next(), Ok(Operation::None));

            // using 3 here are the counter was moved bevore by 1
            assert_eq!(chip.program_counter, curr_pc + 3 * OPCODE_BYTE_SIZE);
        }
    }
}
mod a {
    use super::*;
    #[test]
    fn test_set_index_reg_to_addr() {
        let mut chip = get_default_chip();
        let curr_pc = chip.program_counter;

        let addr = 0x420;
        let opcode: Opcode = 0xA << (3 * 4) ^ addr;

        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

        assert_ne!(chip.index_register, addr);

        assert_eq!(chip.next(), Ok(Operation::None));

        assert_eq!(chip.index_register, addr);

        assert_eq!(chip.program_counter, curr_pc + 1 * OPCODE_BYTE_SIZE);
    }
}
mod b {
    use super::*;
    #[test]
    /// BNNN
    /// Jumps to the address NNN plus V0.
    fn test_jump_to_nnn_with_offset() {
        let mut chip = get_default_chip();

        let offset = 0x10;

        chip.registers[0] = offset;

        let addr = 0x420;
        let opcode: Opcode = 0xB << (3 * 4) ^ addr;

        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

        assert_eq!(chip.next(), Ok(Operation::None));

        assert_eq!(chip.program_counter, (addr + offset as u16) as usize);
    }
}

mod c {
    use super::*;
    use rand::rngs::mock::StepRng;
    #[test]
    /// CXNN
    /// Sets VX to the result of a bitwise and operation on a random number (Typically: 0 to 255)
    /// and NN.
    fn test_bitwise_and_random() {
        let mut chip = get_default_chip();
        // creating a simple "random number generator" that will
        // allways return 0x42 for a simple test.
        let srng = StepRng::new(0x42, 0);
        chip.rng = Box::new(srng);

        let pc = chip.program_counter;

        let base = 0x42;
        let reg = 0x1;
        let anded = 0x20;
        let opcode: Opcode = 0xC << (3 * 4) ^ (reg as u16) << (2 * 4) ^ (anded as u16);

        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

        assert_eq!(chip.next(), Ok(Operation::None));

        assert_eq!(chip.registers[reg as usize], anded & base);

        assert_eq!(chip.program_counter, pc + OPCODE_BYTE_SIZE);
    }
}

mod d {}

mod e {
    use {super::*, crate::definitions::KEYBOARD_SIZE};

    #[test]
    fn test_skip_key_pressed() {
        let rom = get_base();
        let reg1 = 0x1;
        let reg2 = 0x0;

        let mut keyboard = vec![false; KEYBOARD_SIZE].into_boxed_slice();
        keyboard[reg1] = true;

        let mut chip = setup_chip(rom);
        chip.set_keyboard(&keyboard);

        for (i, reg) in [reg2, reg1].iter().enumerate() {
            chip.registers[*reg] = *reg as u8;
            let opcode = 0xE << (3 * 4) ^ (*reg as Opcode) << (2 * 4) ^ 0x9E;

            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            let pc = chip.program_counter;

            assert_eq!(chip.next(), Ok(Operation::None));

            assert_eq!(chip.program_counter, pc + (i + 1) * OPCODE_BYTE_SIZE);
        }
    }

    #[test]
    fn test_skip_key_not_pressed() {
        let rom = get_base();
        let reg1 = 0x0;
        let reg2 = 0x1;

        let mut keyboard = vec![false; KEYBOARD_SIZE].into_boxed_slice();
        keyboard[reg1] = true;

        let mut chip = setup_chip(rom);

        chip.set_keyboard(&keyboard);

        for (i, reg) in [reg1, reg2].iter().enumerate() {
            let pc = chip.program_counter;

            chip.registers[*reg] = *reg as u8;

            let opcode = 0xE << (3 * 4) ^ (*reg as Opcode) << (2 * 4) ^ 0xA1;
            write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

            assert_eq!(chip.next(), Ok(Operation::None));

            assert_eq!(chip.program_counter, pc + (i + 1) * OPCODE_BYTE_SIZE);
        }
    }

    #[test]
    fn test_wrong_opcode() {
        let rom = get_base();
        let reg = 0x0;

        let mut keyboard = vec![false; KEYBOARD_SIZE].into_boxed_slice();
        keyboard[reg] = true;

        let mut chip = setup_chip(rom);
        chip.set_keyboard(&keyboard);

        let pc = chip.program_counter;

        let opcode = 0xE << (3 * 4) ^ (reg as Opcode) << (2 * 4) ^ 0x11;
        write_opcode_to_memory(&mut chip.memory, chip.program_counter, opcode);

        assert_eq!(
            chip.next(),
            Err(format!("An unsupported opcode was used {:#06X?}", opcode))
        );

        assert_eq!(chip.program_counter, pc);
    }
}

mod f {}