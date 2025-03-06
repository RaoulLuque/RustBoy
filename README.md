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

## Implemented Instructions

For a more interactive view,
see [here](https://meganesu.github.io/generate-gb-opcodes/).

```markdown
|    | x0             | x1            | x2             | x3            | x4              | x5            | x6             | x7            | x8              | x9            | xA             | xB          | xC             | xD           | xE             | xF          |
|----|----------------|---------------|----------------|---------------|-----------------|---------------|----------------|---------------|-----------------|---------------|----------------|-------------|----------------|--------------|----------------|-------------|
| 0x | [x] NOP        | [x] LD BC,d16 | [x] LD (BC),A  | [x] INC BC    | [x] INC B       | [x] DEC B     | [x] LD B,d8    | [ ] RLCA      | [x] LD (a16),SP | [x] ADD HL,BC | [x] LD A,(BC)  | [x] DEC BC  | [x] INC C      | [x] DEC C    | [x] LD C,d8    | [ ] RRCA    |
| 1x | [ ] STOP       | [x] LD DE,d16 | [x] LD (DE),A  | [x] INC DE    | [x] INC D       | [x] DEC D     | [x] LD D,d8    | [ ] RLA       | [x] JR r8       | [x] ADD HL,DE | [x] LD A,(DE)  | [x] DEC DE  | [x] INC E      | [x] DEC E    | [x] LD E,d8    | [ ] RRA     |
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
| 0x | [ ] RLC B   | [ ] RLC C   | [ ] RLC D   | [ ] RLC E   | [ ] RLC H   | [ ] RLC L   | [ ] RLC (HL)   | [ ] RLC A   | [ ] RRC B   | [ ] RRC C   | [ ] RRC D   | [ ] RRC E   | [ ] RRC H   | [ ] RRC L   | [ ] RRC (HL)   | [ ] RRC A   |
| 1x | [ ] RL B    | [ ] RL C    | [ ] RL D    | [ ] RL E    | [ ] RL H    | [ ] RL L    | [ ] RL (HL)    | [ ] RL A    | [ ] RR B    | [ ] RR C    | [ ] RR D    | [ ] RR E    | [ ] RR H    | [ ] RR L    | [ ] RR (HL)    | [ ] RR A    |
| 2x | [ ] SLA B   | [ ] SLA C   | [ ] SLA D   | [ ] SLA E   | [ ] SLA H   | [ ] SLA L   | [ ] SLA (HL)   | [ ] SLA A   | [ ] SRA B   | [ ] SRA C   | [ ] SRA D   | [ ] SRA E   | [ ] SRA H   | [ ] SRA L   | [ ] SRA (HL)   | [ ] SRA A   |
| 3x | [ ] SWAP B  | [ ] SWAP C  | [ ] SWAP D  | [ ] SWAP E  | [ ] SWAP H  | [ ] SWAP L  | [ ] SWAP (HL)  | [ ] SWAP A  | [ ] SRL B   | [ ] SRL C   | [ ] SRL D   | [ ] SRL E   | [ ] SRL H   | [ ] SRL L   | [ ] SRL (HL)   | [ ] SRL A   |
| 4x | [ ] BIT 0,B | [ ] BIT 0,C | [ ] BIT 0,D | [ ] BIT 0,E | [ ] BIT 0,H | [ ] BIT 0,L | [ ] BIT 0,(HL) | [ ] BIT 0,A | [ ] BIT 1,B | [ ] BIT 1,C | [ ] BIT 1,D | [ ] BIT 1,E | [ ] BIT 1,H | [ ] BIT 1,L | [ ] BIT 1,(HL) | [ ] BIT 1,A |
| 5x | [ ] BIT 2,B | [ ] BIT 2,C | [ ] BIT 2,D | [ ] BIT 2,E | [ ] BIT 2,H | [ ] BIT 2,L | [ ] BIT 2,(HL) | [ ] BIT 2,A | [ ] BIT 3,B | [ ] BIT 3,C | [ ] BIT 3,D | [ ] BIT 3,E | [ ] BIT 3,H | [ ] BIT 3,L | [ ] BIT 3,(HL) | [ ] BIT 3,A |
| 6x | [ ] BIT 4,B | [ ] BIT 4,C | [ ] BIT 4,D | [ ] BIT 4,E | [ ] BIT 4,H | [ ] BIT 4,L | [ ] BIT 4,(HL) | [ ] BIT 4,A | [ ] BIT 5,B | [ ] BIT 5,C | [ ] BIT 5,D | [ ] BIT 5,E | [ ] BIT 5,H | [ ] BIT 5,L | [ ] BIT 5,(HL) | [ ] BIT 5,A |
| 7x | [ ] BIT 6,B | [ ] BIT 6,C | [ ] BIT 6,D | [ ] BIT 6,E | [ ] BIT 6,H | [ ] BIT 6,L | [ ] BIT 6,(HL) | [ ] BIT 6,A | [ ] BIT 7,B | [ ] BIT 7,C | [ ] BIT 7,D | [ ] BIT 7,E | [ ] BIT 7,H | [ ] BIT 7,L | [ ] BIT 7,(HL) | [ ] BIT 7,A |
| 8x | [ ] RES 0,B | [ ] RES 0,C | [ ] RES 0,D | [ ] RES 0,E | [ ] RES 0,H | [ ] RES 0,L | [ ] RES 0,(HL) | [ ] RES 0,A | [ ] RES 1,B | [ ] RES 1,C | [ ] RES 1,D | [ ] RES 1,E | [ ] RES 1,H | [ ] RES 1,L | [ ] RES 1,(HL) | [ ] RES 1,A |
| 9x | [ ] RES 2,B | [ ] RES 2,C | [ ] RES 2,D | [ ] RES 2,E | [ ] RES 2,H | [ ] RES 2,L | [ ] RES 2,(HL) | [ ] RES 2,A | [ ] RES 3,B | [ ] RES 3,C | [ ] RES 3,D | [ ] RES 3,E | [ ] RES 3,H | [ ] RES 3,L | [ ] RES 3,(HL) | [ ] RES 3,A |
| Ax | [ ] RES 4,B | [ ] RES 4,C | [ ] RES 4,D | [ ] RES 4,E | [ ] RES 4,H | [ ] RES 4,L | [ ] RES 4,(HL) | [ ] RES 4,A | [ ] RES 5,B | [ ] RES 5,C | [ ] RES 5,D | [ ] RES 5,E | [ ] RES 5,H | [ ] RES 5,L | [ ] RES 5,(HL) | [ ] RES 5,A |
| Bx | [ ] RES 6,B | [ ] RES 6,C | [ ] RES 6,D | [ ] RES 6,E | [ ] RES 6,H | [ ] RES 6,L | [ ] RES 6,(HL) | [ ] RES 6,A | [ ] RES 7,B | [ ] RES 7,C | [ ] RES 7,D | [ ] RES 7,E | [ ] RES 7,H | [ ] RES 7,L | [ ] RES 7,(HL) | [ ] RES 7,A |
| Cx | [ ] SET 0,B | [ ] SET 0,C | [ ] SET 0,D | [ ] SET 0.E | [ ] SET 0,H | [ ] SET 0,L | [ ] SET 0,(HL) | [ ] SET 0,A | [ ] SET 1,B | [ ] SET 1,C | [ ] SET 1,D | [ ] SET 1,E | [ ] SET 1,H | [ ] SET 1,L | [ ] SET 1,(HL) | [ ] SET 1,A |
| Dx | [ ] SET 2,B | [ ] SET 2,C | [ ] SET 2,D | [ ] SET 2,E | [ ] SET 2,H | [ ] SET 2,L | [ ] SET 2,(HL) | [ ] SET 2,A | [ ] SET 3,B | [ ] SET 3,C | [ ] SET 3,D | [ ] SET 3.E | [ ] SET 3,H | [ ] SET 3.L | [ ] SET 3,(HL) | [ ] SET 3,A |
| Ex | [ ] SET 4,B | [ ] SET 4,C | [ ] SET 4,D | [ ] SET 4,E | [ ] SET 4,H | [ ] SET 4,L | [ ] SET 4,(HL) | [ ] SET 4,A | [ ] SET 5,B | [ ] SET 5.C | [ ] SET 5,D | [ ] SET 5,E | [ ] SET 5,H | [ ] SET 5,L | [ ] SET 5,(HL) | [ ] SET 5,A |
| Fx | [ ] SET 6,B | [ ] SET 6,C | [ ] SET 6,D | [ ] SET 6,E | [ ] SET 6,H | [ ] SET 6,L | [ ] SET 6,(HL) | [ ] SET 6,A | [ ] SET 7,B | [ ] SET 7,C | [ ] SET 7,D | [ ] SET 7,E | [ ] SET 7,H | [ ] SET 7,L | [ ] SET 7,(HL) | [ ] SET 7,A |
```
