//! The pretty print implementation written for both the  [`internal chipset`](super::InternalChipSet) and the [`external`](super::ChipSet).
//! This implementation was split up into this file for smaller file sizes and higher
//! cohesion.

use super::*;
use crate::{
    definitions::cpu,
    timer::{TimedWorker, TimerCallback},
};
use std::fmt;

impl<W, S> fmt::Display for ChipSet<W, S>
where
    W: TimedWorker,
    S: TimerCallback,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.chipset())
    }
}

/// The length of the pretty print data
/// as a single instruction is u16 the octa
/// size will show how often the block shall
/// be repeated has to be bigger then 0
const HEX_PRINT_STEP: usize = 8;

const END_OF_LINE: char = '\n';
const INDENT_FILLAMENT: char = '\t';
const INDENT_SIZE: usize = 2;

/// Will add an indent post processing
fn indent_helper(text: &mut String, indent: usize) {
    for _ in 0..indent {
        text.push(INDENT_FILLAMENT);
    }
}

macro_rules! intsize {
    () => {
        6
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! intformat {
    () => {
        // The formatted string will be 2 sysbols for the prefix (0x)
        // and 4 for the rest long.
        concat!("{:#0", intsize!(), "X}")
    };
}

const INTSIZE: usize = intsize!();

lazy_static::lazy_static! {
    static ref POINTER_LEN : usize = {
        // create a string that is big enough
        let mut line = String::with_capacity(20);
        // If there was an error panicing here is correct,
        // as some essential component of printing went
        // wrongly.
        pointer_print::formatter(&mut line, 0,0).unwrap();
        line.len()
    };
    static ref INTEGER_LEN : usize = {
        let mut string = String::new();
        // SAFETY: if something went wrong here panicing is correct.
        integer_print::formatter(&mut string, 0u8).unwrap();
        string.len()
    };
    // calculate a line lenght (This is a bit bigger then the actual line will be)
    static ref LENLINE : usize = {
        INDENT_SIZE + HEX_PRINT_STEP * (*INTEGER_LEN + 1) + 1 + *POINTER_LEN
    };
}

/// Handles all the printing of the pointer values.
mod pointer_print {
    use std::fmt::Write;
    /// will formatt the pointers according to definition
    pub(super) fn formatter(
        line: &mut String,
        from: usize,
        to: usize,
    ) -> Result<(), std::fmt::Error> {
        write!(
            line,
            concat!(intformat!(), " - ", intformat!(), " :"),
            from, to
        )
    }
}

/// Handles all the opcode prints
mod opcode_print {
    use super::{integer_print, pointer_print, HEX_PRINT_STEP};
    use crate::{
        definitions::memory,
        opcode::{self, Opcode},
    };
    use std::fmt::{self, Write};

    /// The internal length of the given data
    /// as the data is stored as u8 and an opcode
    /// is u16 long
    const POINTER_INCREMENT: usize = HEX_PRINT_STEP * memory::opcodes::SIZE;
    /// The values that are used when there are at lease two rows of zeros.
    const FILLER_BASE: &str = "...";

    lazy_static::lazy_static! {
        /// Prepares the line that will be used, in the case that there is at least two lines of only zeros.
        static ref ZERO_FILLER : String = {
        // preparing for the 0 block fillers
            let mut formatted = String::new();
            // SAFTY: If there is an error here panicing is correct
            integer_print::formatter(&mut formatted, 0u16).unwrap();
            match HEX_PRINT_STEP {
                1 => formatted,
                2 => format!("{} {}", formatted, formatted),
                _ => {
                    let lenght = formatted.len() * (HEX_PRINT_STEP - 2) + (HEX_PRINT_STEP - 1)
                         - FILLER_BASE.len();
                    let filler = " ".repeat(lenght / 2);

                    format!("{}{}{}{}{}",
                        formatted.clone(),
                        filler.clone(),
                        FILLER_BASE,
                        filler,
                        formatted
                    )
                }
            }
       };
    }

    /// this struct will simulate a single row of opcodes (only in this context)
    struct Row {
        from: usize,
        to: usize,
        data: [Opcode; HEX_PRINT_STEP],
        only_null: bool,
    }

    /// using the fmt::Display` for simple printing of the data later on
    impl fmt::Display for Row {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut res = String::with_capacity(*super::LENLINE);
            pointer_print::formatter(&mut res, self.from, self.to)?;
            res.push(' ');

            if !self.only_null {
                for entry in self.data.iter() {
                    integer_print::formatter(&mut res, *entry)?;
                    res.push(' ');
                }
                if let Some(index) = res.rfind(' ') {
                    res.truncate(index);
                }
            } else {
                res.push_str(&ZERO_FILLER)
            }
            write!(f, "{}", res)
        }
    }

    /// will pretty print the content of the raw memory
    /// this functions assumes the full data to be passed
    /// as the offset is calculated from the beginning of the
    /// memory block
    pub(super) fn printer(memory: &[u8], indent: usize) -> String {
        let data_last_index = memory.len() - 1;
        let mut rows: Vec<Row> = Vec::with_capacity(memory.len() / HEX_PRINT_STEP);

        for from in (0..memory.len()).step_by(POINTER_INCREMENT) {
            // precalculate the end location
            let to = (from + POINTER_INCREMENT - 1).min(data_last_index);

            let mut data = [0; HEX_PRINT_STEP];
            let mut data_index = 0;
            let mut only_null = true;

            // loop over all the opcodes u8 pairs
            for index in (from..=to).step_by(memory::opcodes::SIZE) {
                // set the opcode
                data[data_index] = opcode::build_opcode(memory, index)
                    .expect("Please check if memory is valid in the given Rom.");

                // check if opcode is above 0, if so toggle the is null flag
                if data[data_index] > 0 {
                    only_null = false;
                }
                data_index += 1;
            }

            // create the row that shall be used later on
            let mut row = Row {
                from,
                to,
                data,
                only_null,
            };

            if only_null {
                if let Some(last_row) = rows.last() {
                    if last_row.only_null {
                        row.from = last_row.from;
                        rows.pop();
                    }
                }
            }
            rows.push(row)
        }

        // create the end structure to be used for calculations
        let mut string = String::with_capacity((*super::LENLINE + 1) * rows.len());
        for row in rows {
            super::indent_helper(&mut string, indent);

            if let Err(err) = write!(string, "{}{}", row, super::END_OF_LINE) {
                panic!(err);
            }
        }
        if let Some(index) = string.rfind("\n") {
            string.truncate(index);
        }
        string
    }
}

/// handles printting of any and all of intergers.
mod integer_print {
    use super::{pointer_print, HEX_PRINT_STEP};
    use num;
    use std::fmt::{self, Write};

    /// will format all integer types
    pub(super) fn formatter<T>(line: &mut String, data: T) -> Result<(), fmt::Error>
    where
        T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy,
    {
        write!(line, intformat!(), data)
    }

    /// will pretty print all the integer data given
    pub(super) fn printer<T>(data: &[T], indent: usize) -> Result<String, std::fmt::Error>
    where
        T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy,
    {
        let result_size = *super::LENLINE * (data.len() / HEX_PRINT_STEP);

        let mut res = String::with_capacity(result_size);
        for i in (0..data.len()).step_by(HEX_PRINT_STEP) {
            let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);

            super::indent_helper(&mut res, indent);
            // Copy into the string
            pointer_print::formatter(&mut res, i, n)?;
            res.push(' ');

            for entry in &data[i..=n] {
                if let Err(err) = write!(res, concat!(intformat!(), " "), *entry) {
                    panic!("{}", err);
                }
            }

            // remove unneded whitespace and replace it with a newline
            let index = res.rfind(' ').unwrap();
            res.truncate(index);
            res.push(super::END_OF_LINE);
        }

        // Remove unneded new line
        if let Some(index) = res.rfind('\n') {
            res.truncate(index);
        }

        Ok(res)
    }
}

/// Handles all the boolean data types.
mod bool_print {
    use super::{pointer_print, END_OF_LINE, HEX_PRINT_STEP};

    lazy_static::lazy_static! {
        /// the prepared true string
        static ref TRUE : String = formatter("true");
        /// the prepared false string
        static ref FALSE: String = formatter("false");
    }

    /// a function to keep the correct format length
    fn formatter(message: &str) -> String {
        let mut string = String::with_capacity(*super::INTEGER_LEN);
        string.push_str(message);
        // Fill up the string with information
        while string.len() < *super::INTEGER_LEN {
            string.push(' ');
        }
        string
    }

    /// will pretty print all the boolean data given
    /// the offset will be calculated automatically from
    /// the data block
    pub(super) fn printer(data: &[bool], indent: usize) -> Result<String, std::fmt::Error> {
        let result_size = *super::LENLINE * data.len() / HEX_PRINT_STEP;

        let mut res = String::with_capacity(result_size);

        let check_type = |val: bool| if val { &*TRUE } else { &*FALSE };

        for i in (0..data.len()).step_by(HEX_PRINT_STEP) {
            let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);
            super::indent_helper(&mut res, indent);

            pointer_print::formatter(&mut res, i, n)?;
            res.push(' ');

            for value in &data[i..n] {
                res.push_str(check_type(*value));
                res.push(' ');
            }
            // Append the last missing entry
            res.push_str(check_type(data[n]).trim_end());
            res.push(END_OF_LINE);
        }
        // Remove unneeded new line
        if let Some(index) = res.rfind(END_OF_LINE) {
            res.truncate(index);
        }

        Ok(res)
    }
}

impl fmt::Display for InternalChipSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // prepate the rom name
        let mut nam = String::with_capacity(INDENT_SIZE + self.name.len());
        indent_helper(&mut nam, INDENT_SIZE);
        nam.push_str(&self.name);

        // keeping the strings mutable so that they can be indented later on
        let mem = opcode_print::printer(&self.memory, INDENT_SIZE);
        let reg = integer_print::printer(&self.registers, INDENT_SIZE)?;

        // handle stack specially as it needes to be filled up if empty
        let mut stack = [0; cpu::stack::SIZE];
        stack[0..self.stack.len()].copy_from_slice(&self.stack);

        let sta = integer_print::printer(&stack, INDENT_SIZE)?;
        let key = bool_print::printer(&self.get_keyboard_read().get_keys(), INDENT_SIZE)?;

        let mut opc = String::with_capacity(INTSIZE + INDENT_SIZE);
        indent_helper(&mut opc, INDENT_SIZE);
        // integer_print::formatter(&mut opc, self.opcode_memory[self.program_counter])?;

        let mut prc = String::with_capacity(INTSIZE + INDENT_SIZE);
        indent_helper(&mut prc, INDENT_SIZE);
        integer_print::formatter(&mut prc, self.program_counter)?;

        write!(
            f,
            "Chipset {{\n\
                \tProgram Name :\n{}\n\
                \tOpcode :\n{}\n\
                \tProgram Counter :\n{}\n\
                \tMemory :\n{}\n\
                \tKeybord :\n{}\n\
                \tStack :\n{}\n\
                \tRegister :\n{}\n\
                }}",
            nam, opc, prc, mem, key, sta, reg
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{super::definitions::keyboard, tests};

    // #[test]
    // fn test_indent_helper() {
    //     let text = "some relevant text\nsome more";
    //     let text_expected = "\t\tsome relevant text\n\t\tsome more";
    //     let indent = 2;
    //     let result = super::indent_helper(text, indent);
    //     assert_eq!(&result, text_expected);
    // }

    const OUTPUT_PRINT: &'static str = "\
        Chipset {\n\
            \tProgram Name :\n\
                \t\t15PUZZLE\n\
            \tOpcode :\n\
                \t\t0x0000\n\
            \tProgram Counter :\n\
                \t\t0x0200\n\
            \tMemory :\n\
                \t\t0x0000 - 0x004F : 0x0000                    ...                    0x0000\n\
                \t\t0x0050 - 0x005F : 0xF090 0x9090 0xF020 0x6020 0x2070 0xF010 0xF080 0xF0F0\n\
                \t\t0x0060 - 0x006F : 0x10F0 0x10F0 0x9090 0xF010 0x10F0 0x80F0 0x10F0 0xF080\n\
                \t\t0x0070 - 0x007F : 0xF090 0xF0F0 0x1020 0x4040 0xF090 0xF090 0xF0F0 0x90F0\n\
                \t\t0x0080 - 0x008F : 0x10F0 0xF090 0xF090 0x90E0 0x90E0 0x90E0 0xF080 0x8080\n\
                \t\t0x0090 - 0x009F : 0xF0E0 0x9090 0x90E0 0xF080 0xF080 0xF0F0 0x80F0 0x8080\n\
                \t\t0x00A0 - 0x01FF : 0x0000                    ...                    0x0000\n\
                \t\t0x0200 - 0x020F : 0x00E0 0x6C00 0x4C00 0x6E0F 0xA203 0x6020 0xF055 0x00E0\n\
                \t\t0x0210 - 0x021F : 0x22BE 0x2276 0x228E 0x225E 0x2246 0x1210 0x6100 0x6217\n\
                \t\t0x0220 - 0x022F : 0x6304 0x4110 0x00EE 0xA2E8 0xF11E 0xF065 0x4000 0x1234\n\
                \t\t0x0230 - 0x023F : 0xF029 0xD235 0x7101 0x7205 0x6403 0x8412 0x3400 0x1222\n\
                \t\t0x0240 - 0x024F : 0x6217 0x7306 0x1222 0x6403 0x84E2 0x6503 0x85D2 0x9450\n\
                \t\t0x0250 - 0x025F : 0x00EE 0x4403 0x00EE 0x6401 0x84E4 0x22A6 0x1246 0x6403\n\
                \t\t0x0260 - 0x026F : 0x84E2 0x6503 0x85D2 0x9450 0x00EE 0x4400 0x00EE 0x64FF\n\
                \t\t0x0270 - 0x027F : 0x84E4 0x22A6 0x125E 0x640C 0x84E2 0x650C 0x85D2 0x9450\n\
                \t\t0x0280 - 0x028F : 0x00EE 0x4400 0x00EE 0x64FC 0x84E4 0x22A6 0x1276 0x640C\n\
                \t\t0x0290 - 0x029F : 0x84E2 0x650C 0x85D2 0x9450 0x00EE 0x440C 0x00EE 0x6404\n\
                \t\t0x02A0 - 0x02AF : 0x84E4 0x22A6 0x128E 0xA2E8 0xF41E 0xF065 0xA2E8 0xFE1E\n\
                \t\t0x02B0 - 0x02BF : 0xF055 0x6000 0xA2E8 0xF41E 0xF055 0x8E40 0x00EE 0x3C00\n\
                \t\t0x02C0 - 0x02CF : 0x12D2 0x221C 0x22D8 0x221C 0xA2F8 0xFD1E 0xF065 0x8D00\n\
                \t\t0x02D0 - 0x02DF : 0x00EE 0x7CFF 0xCD0F 0x00EE 0x7D01 0x600F 0x8D02 0xED9E\n\
                \t\t0x02E0 - 0x02EF : 0x12D8 0xEDA1 0x12E2 0x00EE 0x0102 0x0304 0x0506 0x0708\n\
                \t\t0x02F0 - 0x02FF : 0x090A 0x0B0C 0x0D0E 0x0F00 0x0D00 0x0102 0x0405 0x0608\n\
                \t\t0x0300 - 0x030F : 0x090A 0x0C0E 0x0307 0x0B0F 0x84E4 0x22A6 0x1276 0x640C\n\
                \t\t0x0310 - 0x031F : 0x84E2 0x650C 0x85D2 0x9450 0x00EE 0x440C 0x00EE 0x6404\n\
                \t\t0x0320 - 0x032F : 0x84E4 0x22A6 0x128E 0xA2E8 0xF41E 0xF065 0xA2E8 0xFE1E\n\
                \t\t0x0330 - 0x033F : 0xF055 0x6000 0xA2E8 0xF41E 0xF055 0x8E40 0x00EE 0x3C00\n\
                \t\t0x0340 - 0x034F : 0x12D2 0x221C 0x22D8 0x221C 0xA2F8 0xFD1E 0xF065 0x8D00\n\
                \t\t0x0350 - 0x035F : 0x00EE 0x7CFF 0xCD0F 0x00EE 0x7D01 0x600F 0x8D02 0xED9E\n\
                \t\t0x0360 - 0x036F : 0x12D8 0xEDA1 0x12E2 0x00EE 0x0102 0x0304 0x0506 0x0708\n\
                \t\t0x0370 - 0x037F : 0x090A 0x0B0C 0x0D0E 0x0F00 0x0D00 0x0102 0x0405 0x0608\n\
                \t\t0x0380 - 0x0FFF : 0x0000                    ...                    0x0000\n\
            \tKeybord :\n\
            \t\t0x0000 - 0x0007 : false  true   false  true   false  true   false  true\n\
            \t\t0x0008 - 0x000F : false  true   false  true   false  true   false  true\n\
            \tStack :\n\
                \t\t0x0000 - 0x0007 : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000\n\
                \t\t0x0008 - 0x000F : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000\n\
            \tRegister :\n\
            \t\t0x0000 - 0x0007 : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000\n\
            \t\t0x0008 - 0x000F : 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000 0x0000\n\
        }";

    #[test]
    /// tests if the pretty print output is as expected
    /// this test is mainly for coverage purposes, as
    /// the given module takes up a multitude of lines.
    fn test_full_print() {
        let mut chipset = tests::get_default_chip();
        let chip = chipset.chipset_mut();
        let mut keys = [false; keyboard::SIZE];

        for (index, key) in keys.iter_mut().enumerate() {
            *key = index % 2 != 0;
        }

        chip.set_keyboard(&keys);

        // override the chip register as they are generated randomly
        chip.registers.fill(0);

        let actual_full = format!("{}", chip);
        let actual_split = actual_full.split("\n");
        let expected = OUTPUT_PRINT.split("\n");

        for (exp, act) in expected.zip(actual_split) {
            assert_eq!(exp, act);
        }
    }
}
