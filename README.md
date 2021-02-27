# Chip8
CHIP-8 is an interpreted programming language, developed by Joseph Weisbecker. It was initially used on the COSMAC VIP and Telmac 1800 8-bit microcomputers in the mid-1970s. CHIP-8 programs are run on a CHIP-8 virtual machine. It was made to allow video games to be more easily programmed for these computers.

Roughly fifteen years after CHIP-8 was introduced, derived interpreters appeared for some models of graphing calculators (from the late 1980s onward, these handheld devices in many ways have more computing power than most mid-1970s microcomputers for hobbyists).

An active community of users and developers existed in the late 1970s, beginning with ARESCO's "VIPer" newsletter whose first three issues revealed the machine code behind the CHIP-8 interpreter.
see [Chip8](https://en.wikipedia.org/wiki/CHIP-8) for more information

## Project 
This is a project I am working on on the side. It's for me to learn Rust as well as the Chip8 infrastructure and of course 
it is meant to teach myself cpu / language architecture.

## Keyboard Layout:

### Chip8 Keypad:
|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

### Emulator Keyboard Mapping:
|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | 4 |
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

'esc' Key  : Close the Emulator<br>
'Spacebar' : Pause / Resume the Emulator<br>
'F5 Key'   : Reset the emulator

## Mnemonic 
Here is the CHIP-8 instructions. [see here](https://massung.github.io/CHIP-8/)

| Opcode|Mnemonic 	   |  Description                                                                           |
|-------|--------------|----------------------------------------------------------------------------------------|
| 00E0 	|CLS    	   |  Clear video memory                                                                    |
| 00EE 	|RET 	       |  Return from subroutine                                                                |
| 0NNN 	|SYS NNN 	   |  Call CDP1802 subroutine at NNN                                                        |
| 2NNN 	|CALL NNN 	   |  Call CHIP-8 subroutine at NNN                                                         |
| 1NNN 	|JP NNN 	   |  Jump to address NNN                                                                   |
| BNNN 	|JP V0, NNN    |  Jump to address NNN + V0                                                              |
| 3XNN 	|SE VX, NN     |  Skip next instruction if VX == NN                                                     |
| 4XNN 	|SNE VX, NN    |  Skip next instruction if VX != NN                                                     |
| 5XY0 	|SE VX, VY 	   |  Skip next instruction if VX == VY                                                     |
| 9XY0 	|SNE VX, VY    |  Skip next instruction if VX != VY                                                     |
| EX9E 	|SKP VX        |  Skip next instruction if key(VX) is pressed                                           |
| EXA1 	|SKNP VX       |   Skip next instruction if key(VX) is not pressed                                      |
| FX0A 	|LD VX, K      |  Wait for key press, store key pressed in VX                                           |
| 6XNN 	|LD VX, NN     |  VX = NN                                                                               |
| 8XY0 	|LD VX, VY     |  VX = VY                                                                               |
| FX07 	|LD VX, DT     |  VX = DT                                                                               |
| FX15 	|LD DT, VX     |  DT = VX                                                                               |
| FX18 	|LD ST, VX     |  ST = VX                                                                               |
| ANNN 	|LD I, NNN     |  I = NNN                                                                               |
| FX29 	|LD F, VX 	   |  I = address of 4x5 font character in VX (0..F) (* see note)                           |
| FX55 	|LD [I], VX    |  Store V0..VX (inclusive) to memory starting at I; I remains unchanged                 |
| FX65 	|LD VX, [I]    |  Load V0..VX (inclusive) from memory starting at I; I remains unchanged                |
| FX1E 	|ADD I, VX 	   |  I = I + VX; VF = 1 if I > 0xFFF else 0                                                |
| 7XNN 	|ADD VX, NN    |  VX = VX + NN                                                                          |
| 8XY4 	|ADD VX, VY    |  VX = VX + VY; VF = 1 if overflow else 0                                               |
| 8XY5 	|SUB VX, VY    |  VX = VX - VY; VF = 1 if not borrow else 0                                             |
| 8XY7 	|SUBN VX, VY   |  VX = VY - VX; VF = 1 if not borrow else 0                                             |
| 8XY1 	|OR VX, VY 	   |  VX = VX OR VY                                                                         |
| 8XY2 	|AND VX, VY    |  VX = VX AND VY                                                                        |
| 8XY3 	|XOR VX, VY    |  VX = VX XOR VY                                                                        |
| 8XY6 	|SHR VX 	   |  VF = LSB(VX); VX = VX » 1 (** see note)                                               |
| 8XYE 	|SHL VX 	   |  VF = MSB(VX); VX = VX « 1 (** see note)                                               |
| FX33 	|BCD VX        |  Store BCD representation of VX at I (100), I+1 (10), and I+2 (1); I remains unchanged |
| CXNN 	|RND VX, NN    |  VX = RND() AND NN                                                                     |
| DXYN 	|DRW VX, VY, N |  Draw 8xN sprite at I to VX, VY; VF = 1 if collision else 0                            |

