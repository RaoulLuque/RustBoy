# RustBoy

A Game Boy Emulator written in Rust using wgpu, wgsl, and winit.
The app also runs in the web using WASM.

## Resources and Tutorials Used

- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/)
- [Pan Docs](https://gbdev.io/pandocs/)
- [RGBDS: gbz80(7) - CPU opcode reference](https://rgbds.gbdev.io/docs/v0.9.0/gbz80.7)
- [Interactive GB Opcode Table](https://meganesu.github.io/generate-gb-opcodes/)
- [Gameboy Doctor](https://github.com/robert/gameboy-doctor)
- [GameBoy Emulation in JavaScript](https://imrannazar.com/series/gameboy-emulation-in-javascript)
- [DMG-01: How to Emulate a Game Boy](https://rylev.github.io/DMG-01/public/book/introduction.html)

## How to Start

TBD
TLDR:

```
RUST_BACKTRACE=1 RUST_LOG=info cargo run -- --SB --ROM "roms/[ROM_NAME].gb"
```

## Implemented Instructions

For a more interactive view,
see [here](https://meganesu.github.io/generate-gb-opcodes/).

```markdown
|    | x0             | x1            | x2             | x3            | x4              | x5            | x6             | x7            | x8              | x9            | xA             | xB          | xC             | xD           | xE             | xF          |
|----|----------------|---------------|----------------|---------------|-----------------|---------------|----------------|---------------|-----------------|---------------|----------------|-------------|----------------|--------------|----------------|-------------|
| 0x | [x] NOP        | [x] LD BC,d16 | [x] LD (BC),A  | [x] INC BC    | [x] INC B       | [x] DEC B     | [x] LD B,d8    | [x] RLCA      | [x] LD (a16),SP | [x] ADD HL,BC | [x] LD A,(BC)  | [x] DEC BC  | [x] INC C      | [x] DEC C    | [x] LD C,d8    | [x] RRCA    |
| 1x | [ ] STOP       | [x] LD DE,d16 | [x] LD (DE),A  | [x] INC DE    | [x] INC D       | [x] DEC D     | [x] LD D,d8    | [x] RLA       | [x] JR r8       | [x] ADD HL,DE | [x] LD A,(DE)  | [x] DEC DE  | [x] INC E      | [x] DEC E    | [x] LD E,d8    | [x] RRA     |
| 2x | [x] JR NZ,r8   | [x] LD HL,d16 | [x] LD (HL+),A | [x] INC HL    | [x] INC H       | [x] DEC H     | [x] LD H,d8    | [x] DAA       | [x] JR Z,r8     | [x] ADD HL,HL | [x] LD A,(HL+) | [x] DEC HL  | [x] INC L      | [x] DEC L    | [x] LD L,d8    | [x] CPL     |
| 3x | [x] JR NC,r8   | [x] LD SP,d16 | [x] LD (HL-),A | [x] INC SP    | [x] INC (HL)    | [x] DEC (HL)  | [x] LD (HL),d8 | [x] SCF       | [x] JR C,r8     | [x] ADD HL,SP | [x] LD A,(HL-) | [x] DEC SP  | [x] INC A      | [x] DEC A    | [x] LD A,d8    | [x] CCF     |
| 4x | [x] LD B,B     | [x] LD B,C    | [x] LD B,D     | [x] LD B,E    | [x] LD B,H      | [x] LD B,L    | [x] LD B,(HL)  | [x] LD B,A    | [x] LD C,B      | [x] LD C,C    | [x] LD C,D     | [x] LD C,E  | [x] LD C,H     | [x] LD C,L   | [x] LD C,(HL)  | [x] LD C,A  |
| 5x | [x] LD D,B     | [x] LD D,C    | [x] LD D,D     | [x] LD D,E    | [x] LD D,H      | [x] LD D,L    | [x] LD D,(HL)  | [x] LD D,A    | [x] LD E,B      | [x] LD E,C    | [x] LD E,D     | [x] LD E,E  | [x] LD E,H     | [x] LD E,L   | [x] LD E,(HL)  | [x] LD E,A  |
| 6x | [x] LD H,B     | [x] LD H,C    | [x] LD H,D     | [x] LD H,E    | [x] LD H,H      | [x] LD H,L    | [x] LD H,(HL)  | [x] LD H,A    | [x] LD L,B      | [x] LD L,C    | [x] LD L,D     | [x] LD L,E  | [x] LD L,H     | [x] LD L,L   | [x] LD L,(HL)  | [x] LD L,A  |
| 7x | [x] LD (HL),B  | [x] LD (HL),C | [x] LD (HL),D  | [x] LD (HL),E | [x] LD (HL),H   | [x] LD (HL),L | [ ] HALT       | [x] LD (HL),A | [x] LD A,B      | [x] LD A,C    | [x] LD A,D     | [x] LD A,E  | [x] LD A,H     | [x] LD A,L   | [x] LD A,(HL)  | [x] LD A,A  |
| 8x | [x] ADD A,B    | [x] ADD A,C   | [x] ADD A,D    | [x] ADD A,E   | [x] ADD A,H     | [x] ADD A,L   | [x] ADD A,(HL) | [x] ADD A,A   | [x] ADC A,B     | [x] ADC A,C   | [x] ADC A,D    | [x] ADC A,E | [x] ADC A,H    | [x] ADC A,L  | [x] ADC A,(HL) | [x] ADC A,A |
| 9x | [x] SUB B      | [x] SUB C     | [x] SUB D      | [x] SUB E     | [x] SUB H       | [x] SUB L     | [x] SUB (HL)   | [x] SUB A     | [x] SBC A,B     | [x] SBC A,C   | [x] SBC A,D    | [x] SBC A,E | [x] SBC A,H    | [x] SBC A,L  | [x] SBC A,(HL) | [x] SBC A,A |
| Ax | [x] AND B      | [x] AND C     | [x] AND D      | [x] AND E     | [x] AND H       | [x] AND L     | [x] AND (HL)   | [x] AND A     | [x] XOR B       | [x] XOR C     | [x] XOR D      | [x] XOR E   | [x] XOR H      | [x] XOR L    | [x] XOR (HL)   | [x] XOR A   |
| Bx | [x] OR B       | [x] OR C      | [x] OR D       | [x] OR E      | [x] OR H        | [x] OR L      | [x] OR (HL)    | [x] OR A      | [x] CP B        | [x] CP C      | [x] CP D       | [x] CP E    | [x] CP H       | [x] CP L     | [x] CP (HL)    | [x] CP A    |
| Cx | [x] RET NZ     | [x] POP BC    | [x] JP NZ,a16  | [x] JP a16    | [x] CALL NZ,a16 | [x] PUSH BC   | [x] ADD A,d8   | [x] RST 0     | [x] RET Z       | [x] RET       | [x] JP Z,a16   |             | [x] CALL Z,a16 | [x] CALL a16 | [x] ADC A,d8   | [x] RST 1   |
| Dx | [x] RET NC     | [x] POP DE    | [x] JP NC,a16  |               | [x] CALL NC,a16 | [x] PUSH DE   | [x] SUB A,d8   | [x] RST 2     | [x] RET C       | [ ] RETI      | [x] JP C,a16   |             | [x] CALL C,a16 |              | [x] SBC A,d8   | [x] RST 3   |
| Ex | [x] LDH (a8),A | [x] POP HL    | [x] LDH (C),A  |               |                 | [x] PUSH HL   | [x] AND d8     | [x] RST 4     | [x] ADD SP,r8   | [x] JP (HL)   | [x] LD (a16),A |             |                |              | [x] XOR d8     | [x] RST 5   |
| Fx | [x] LDH A,(a8) | [x] POP AF    | [x] LDH A,(C)  | [x] DI        |                 | [x] PUSH AF   | [x] OR d8      | [x] RST 6     | [x] LD HL,SP+r8 | [x] LD SP,HL  | [x] LD A,(a16) | [x] EI      |                |              | [x] CP d8      | [x] RST 7   |
```

```markdown
|    | x0          | x1          | x2          | x3          | x4          | x5          | x6             | x7          | x8          | x9          | xA          | xB          | xC          | xD          | xE             | xF          |
|----|-------------|-------------|-------------|-------------|-------------|-------------|----------------|-------------|-------------|-------------|-------------|-------------|-------------|-------------|----------------|-------------|
| 0x | [x] RLC B   | [x] RLC C   | [x] RLC D   | [x] RLC E   | [x] RLC H   | [x] RLC L   | [x] RLC (HL)   | [x] RLC A   | [x] RRC B   | [x] RRC C   | [x] RRC D   | [x] RRC E   | [x] RRC H   | [x] RRC L   | [x] RRC (HL)   | [x] RRC A   |
| 1x | [x] RL B    | [x] RL C    | [x] RL D    | [x] RL E    | [x] RL H    | [x] RL L    | [x] RL (HL)    | [x] RL A    | [x] RR B    | [x] RR C    | [x] RR D    | [x] RR E    | [x] RR H    | [x] RR L    | [x] RR (HL)    | [x] RR A    |
| 2x | [x] SLA B   | [x] SLA C   | [x] SLA D   | [x] SLA E   | [x] SLA H   | [x] SLA L   | [x] SLA (HL)   | [x] SLA A   | [x] SRA B   | [x] SRA C   | [x] SRA D   | [x] SRA E   | [x] SRA H   | [x] SRA L   | [x] SRA (HL)   | [x] SRA A   |
| 3x | [x] SWAP B  | [x] SWAP C  | [x] SWAP D  | [x] SWAP E  | [x] SWAP H  | [x] SWAP L  | [x] SWAP (HL)  | [x] SWAP A  | [x] SRL B   | [x] SRL C   | [x] SRL D   | [x] SRL E   | [x] SRL H   | [x] SRL L   | [x] SRL (HL)   | [x] SRL A   |
| 4x | [x] BIT 0,B | [x] BIT 0,C | [x] BIT 0,D | [x] BIT 0,E | [x] BIT 0,H | [x] BIT 0,L | [x] BIT 0,(HL) | [x] BIT 0,A | [x] BIT 1,B | [x] BIT 1,C | [x] BIT 1,D | [x] BIT 1,E | [x] BIT 1,H | [x] BIT 1,L | [x] BIT 1,(HL) | [x] BIT 1,A |
| 5x | [x] BIT 2,B | [x] BIT 2,C | [x] BIT 2,D | [x] BIT 2,E | [x] BIT 2,H | [x] BIT 2,L | [x] BIT 2,(HL) | [x] BIT 2,A | [x] BIT 3,B | [x] BIT 3,C | [x] BIT 3,D | [x] BIT 3,E | [x] BIT 3,H | [x] BIT 3,L | [x] BIT 3,(HL) | [x] BIT 3,A |
| 6x | [x] BIT 4,B | [x] BIT 4,C | [x] BIT 4,D | [x] BIT 4,E | [x] BIT 4,H | [x] BIT 4,L | [x] BIT 4,(HL) | [x] BIT 4,A | [x] BIT 5,B | [x] BIT 5,C | [x] BIT 5,D | [x] BIT 5,E | [x] BIT 5,H | [x] BIT 5,L | [x] BIT 5,(HL) | [x] BIT 5,A |
| 7x | [x] BIT 6,B | [x] BIT 6,C | [x] BIT 6,D | [x] BIT 6,E | [x] BIT 6,H | [x] BIT 6,L | [x] BIT 6,(HL) | [x] BIT 6,A | [x] BIT 7,B | [x] BIT 7,C | [x] BIT 7,D | [x] BIT 7,E | [x] BIT 7,H | [x] BIT 7,L | [x] BIT 7,(HL) | [x] BIT 7,A |
| 8x | [x] RES 0,B | [x] RES 0,C | [x] RES 0,D | [x] RES 0,E | [x] RES 0,H | [x] RES 0,L | [x] RES 0,(HL) | [x] RES 0,A | [x] RES 1,B | [x] RES 1,C | [x] RES 1,D | [x] RES 1,E | [x] RES 1,H | [x] RES 1,L | [x] RES 1,(HL) | [x] RES 1,A |
| 9x | [x] RES 2,B | [x] RES 2,C | [x] RES 2,D | [x] RES 2,E | [x] RES 2,H | [x] RES 2,L | [x] RES 2,(HL) | [x] RES 2,A | [x] RES 3,B | [x] RES 3,C | [x] RES 3,D | [x] RES 3,E | [x] RES 3,H | [x] RES 3,L | [x] RES 3,(HL) | [x] RES 3,A |
| Ax | [x] RES 4,B | [x] RES 4,C | [x] RES 4,D | [x] RES 4,E | [x] RES 4,H | [x] RES 4,L | [x] RES 4,(HL) | [x] RES 4,A | [x] RES 5,B | [x] RES 5,C | [x] RES 5,D | [x] RES 5,E | [x] RES 5,H | [x] RES 5,L | [x] RES 5,(HL) | [x] RES 5,A |
| Bx | [x] RES 6,B | [x] RES 6,C | [x] RES 6,D | [x] RES 6,E | [x] RES 6,H | [x] RES 6,L | [x] RES 6,(HL) | [x] RES 6,A | [x] RES 7,B | [x] RES 7,C | [x] RES 7,D | [x] RES 7,E | [x] RES 7,H | [x] RES 7,L | [x] RES 7,(HL) | [x] RES 7,A |
| Cx | [x] SET 0,B | [x] SET 0,C | [x] SET 0,D | [x] SET 0.E | [x] SET 0,H | [x] SET 0,L | [x] SET 0,(HL) | [x] SET 0,A | [x] SET 1,B | [x] SET 1,C | [x] SET 1,D | [x] SET 1,E | [x] SET 1,H | [x] SET 1,L | [x] SET 1,(HL) | [x] SET 1,A |
| Dx | [x] SET 2,B | [x] SET 2,C | [x] SET 2,D | [x] SET 2,E | [x] SET 2,H | [x] SET 2,L | [x] SET 2,(HL) | [x] SET 2,A | [x] SET 3,B | [x] SET 3,C | [x] SET 3,D | [x] SET 3.E | [x] SET 3,H | [x] SET 3.L | [x] SET 3,(HL) | [x] SET 3,A |
| Ex | [x] SET 4,B | [x] SET 4,C | [x] SET 4,D | [x] SET 4,E | [x] SET 4,H | [x] SET 4,L | [x] SET 4,(HL) | [x] SET 4,A | [x] SET 5,B | [x] SET 5,C | [x] SET 5,D | [x] SET 5,E | [x] SET 5,H | [x] SET 5,L | [x] SET 5,(HL) | [x] SET 5,A |
| Fx | [x] SET 6,B | [x] SET 6,C | [x] SET 6,D | [x] SET 6,E | [x] SET 6,H | [x] SET 6,L | [x] SET 6,(HL) | [x] SET 6,A | [x] SET 7,B | [x] SET 7,C | [x] SET 7,D | [x] SET 7,E | [x] SET 7,H | [x] SET 7,L | [x] SET 7,(HL) | [x] SET 7,A |
```
