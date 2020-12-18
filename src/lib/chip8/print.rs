use {
    super::{ChipSet, DisplayCommands, KeyboardCommands},
    std::fmt,
};

/// The length of the pretty print data
/// as a single instruction is u16 the octa
/// size will show how often the block shall
/// be repeated has to be bigger then 0
const HEX_PRINT_STEP: usize = 8;

/// will add an indent post processing
///
/// Example
fn indent_helper(text: &str, indent: usize) -> String {
    let indent = "\t".repeat(indent);
    text.split("\n")
        .map(|x| format!("{}{}\n", indent, x))
        .collect::<String>()
        .trim_end()
        .to_string()
}

mod pointer_print {
    use super::integer_print;
    /// will formatt the pointers according to definition
    pub(super) fn formatter(from: usize, to: usize) -> String {
        format!(
            "{} - {} :",
            integer_print::formatter(from),
            integer_print::formatter(to)
        )
    }
}

mod opcode_print {
    use {
        super::{integer_print, pointer_print, HEX_PRINT_STEP},
        crate::{
            definitions::OPCODE_BYTE_SIZE,
            opcode::{self, Opcode},
        },
        lazy_static,
        std::fmt,
    };

    /// The internal length of the given data
    /// as the data is stored as u8 and an opcode
    /// is u16 long
    const POINTER_INCREMENT: usize = HEX_PRINT_STEP * OPCODE_BYTE_SIZE;

    lazy_static::lazy_static! {
        // preparing for the 0 block fillers
        static ref ZERO_FILLER : String = {
            let formatted = integer_print::formatter(0u16);
            match HEX_PRINT_STEP {
                1 => formatted,
                2 => vec![formatted; 2].join(" "),
                _ => {
                    let filler_base = "...";
                    let lenght = formatted.len() * (HEX_PRINT_STEP - 2) + (HEX_PRINT_STEP - 1)
                         - filler_base.len();
                    let filler = " ".repeat(lenght / 2);
                    format!("{}{}{}{}{}",
                        formatted.clone(),
                        filler.clone(),
                        filler_base,
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
            let mut res = Vec::with_capacity(HEX_PRINT_STEP + 1);
            res.push(pointer_print::formatter(self.from, self.to));

            if !self.only_null {
                for entry in self.data.iter() {
                    res.push(integer_print::formatter(*entry));
                }
            } else {
                res.push(ZERO_FILLER.clone());
            }
            write!(f, "{}", res.join(" "))
        }
    }

    /// will pretty print the content of the raw memory
    /// this functions assumes the full data to be passed
    /// as the offset is calculated from the beginning of the
    /// memory block
    pub(super) fn printer(memory: &[u8], offset: usize) -> String {
        // using the offset
        let data_last_index = memory.len() - 1;
        let mut rows: Vec<Row> = Vec::with_capacity((memory.len() - offset) / HEX_PRINT_STEP);

        for from in (offset..memory.len()).step_by(POINTER_INCREMENT) {
            // precalculate the end location
            let to = (from + POINTER_INCREMENT - 1).min(data_last_index);

            let mut data = [0; HEX_PRINT_STEP];
            let mut data_index = 0;
            let mut only_null = true;

            // loop over all the opcodes u8 pairs
            for index in (from..=to).step_by(OPCODE_BYTE_SIZE) {
                // set the opcode
                data[data_index] = opcode::build_opcode(memory, index);

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
        rows.iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

mod integer_print {
    use {
        super::{pointer_print, HEX_PRINT_STEP},
        num,
        std::fmt,
    };
    /// will format all integer types
    pub(super) fn formatter<T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy>(
        data: T,
    ) -> String {
        format!("{:#06X}", data)
    }

    /// will pretty print all the integer data given
    pub(super) fn printer<T: fmt::Display + fmt::UpperHex + num::Unsigned + Copy>(
        data: &[T],
        offset: usize,
    ) -> String {
        let mut res = Vec::new();
        for i in (offset..data.len()).step_by(HEX_PRINT_STEP) {
            let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);
            let mut row = vec![pointer_print::formatter(i, n)];

            for j in i..=n {
                row.push(formatter(data[j]));
            }
            res.push(row.join(" "));
        }
        res.join("\n")
    }
}

mod bool_print {
    use {
        super::{integer_print, pointer_print, HEX_PRINT_STEP},
        lazy_static,
    };

    lazy_static::lazy_static! {
        static ref TRUE : String = formatter("true");
        static ref FALSE: String = formatter("false");
    }

    /// a function to keep the correct format length
    pub(super) fn formatter(string: &str) -> String {
        let mut string = string.to_string();
        let formatted = integer_print::formatter(0u16);
        while string.len() < formatted.len() {
            string.push(' ');
        }
        string
    }

    /// will pretty print all the boolean data given
    /// the offset will be calculated automatically from
    /// the data block
    pub(super) fn printer(data: &[bool], offset: usize) -> String {
        let mut res = Vec::new();

        for i in (offset..data.len()).step_by(HEX_PRINT_STEP) {
            let n = (i + HEX_PRINT_STEP - 1).min(data.len() - 1);
            let mut row = vec![pointer_print::formatter(i, n)];

            for j in i..=n {
                row.push(if data[j] { TRUE.clone() } else { FALSE.clone() });
            }
            res.push(row.join(" ").trim_end().to_string());
        }
        res.join("\n")
    }
}

impl<T: DisplayCommands, U: KeyboardCommands> fmt::Display for ChipSet<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // keeping the strings mutable so that they can be indented later on
        let mut mem = opcode_print::printer(&self.memory, 0);
        let mut reg = integer_print::printer(&self.registers, 0);
        let mut sta = integer_print::printer(&self.stack, 0);
        let mut key = bool_print::printer(&self.keyboard.get_keyboard(), 0);

        let mut opc = integer_print::formatter(self.opcode);
        let mut prc = integer_print::formatter(self.program_counter);
        let mut stc = integer_print::formatter(self.stack_pointer);

        // using a mutable slice here for convenient iterating
        let mut data = [
            &mut mem, &mut reg, &mut key, &mut sta, &mut opc, &mut prc, &mut stc,
        ];

        for string in data.iter_mut() {
            **string = indent_helper(string, 2);
        }

        write!(
            f,
            "Chipset {{\n\
                \tProgram Name: {}\n\
                \tOpcode :\n{}\n\
                \tProgram Counter:\n{}\n\
                \tMemory :\n{}\n\
                \tKeybord :\n{}\n\
                \tStack Pointer :\n{}\n\
                \tStack :\n{}\n\
                \tRegister :\n{}\n\
                }}",
            self.name, opc, prc, mem, key, stc, sta, reg
        )
    }
}

#[cfg(test)]
mod tests {
    use {
        super::super::super::definitions::{KEYBOARD_SIZE, REGISTER_SIZE},
        super::super::tests::*,
        super::*,
    };

    #[test]
    fn test_indent_helper() {
        let text = "some relevant text\nsome more";
        let text_expected = "\t\tsome relevant text\n\t\tsome more";
        let indent = 2;
        let result = indent_helper(text, indent);
        assert_eq!(&result, text_expected);
    }

    const OUTPUT_PRINT: &str = "\
    Chipset {\n\
        \tProgram Name: 15PUZZLE\n\
        \tOpcode :\n\
        \t\t0x0000\n\
        \tProgram Counter:\n\
        \t\t0x0200\n\
        \tMemory :\n\
            \t\t0x0000 - 0x000F : 0xF090 0x9090 0xF020 0x6020 0x2070 0xF010 0xF080 0xF0F0\n\
            \t\t0x0010 - 0x001F : 0x10F0 0x10F0 0x9090 0xF010 0x10F0 0x80F0 0x10F0 0xF080\n\
            \t\t0x0020 - 0x002F : 0xF090 0xF0F0 0x1020 0x4040 0xF090 0xF090 0xF0F0 0x90F0\n\
            \t\t0x0030 - 0x003F : 0x10F0 0xF090 0xF090 0x90E0 0x90E0 0x90E0 0xF080 0x8080\n\
            \t\t0x0040 - 0x004F : 0xF0E0 0x9090 0x90E0 0xF080 0xF080 0xF0F0 0x80F0 0x8080\n\
            \t\t0x0050 - 0x01FF : 0x0000                    ...                    0x0000\n\
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
        \tStack Pointer :\n\
            \t\t0x0000\n\
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
        let (rom, dis, mut key) = get_base();
        let keys = (0..KEYBOARD_SIZE)
            .map(|i| i % 2 != 0)
            .collect::<Vec<bool>>()
            .into_boxed_slice();
        key.expect_get_keyboard().returning(move || keys.clone());
        let mut chip = setup_chip(rom, dis, key);

        // override the chip register as they are generated randomly

        chip.registers = (0..REGISTER_SIZE).map(|_| 0 as u8).collect();
        assert_eq!(format!("{}", chip), OUTPUT_PRINT);
    }
}
