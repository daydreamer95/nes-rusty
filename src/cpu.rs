use crate::ops_code::OPCODES_MAP;
use crate::ops_code::Opcode;
use std::collections::HashMap;

const MEMORY_SIZE: u16 = 0xFFFF;
const PROGRAM_ROM_MEMORY_ADDRESS_START: u16 = 0x0600;
const RESET_INTERRUPT_ADDR: u16 = 0xFFFC;
const STACK_STARTING_POINTER: u8 = 0xFF;

#[derive(Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Accumulator,
    Relative,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX, // Absolute, X
    AbsoluteY, // Absolute, Y
    Indirect,
    IndirectX, // (Indirect, X)
    IndirectY, // (Indirect), Y
    NoneAddressing,
}

#[derive(Debug, PartialEq, Eq)]
enum MathematicalOperation {
    Add,
    Sub,
}

// Cpu Represent the 6502 CPU version used by NES.
#[derive(Debug, PartialEq, Eq)]
pub struct CPU {
    pub accumulator: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    // flags. The flags is nothing more than 1-bit values that tells us the current state of
    // the operations happening in the CPU. In repsective order they are then
    // #76543210
    // 7 - Negative result
    // 6 - Overflow
    // 4 - Break
    // 3 - Decimal mode
    // 2 - Interupt disable
    // 1 - Zero result
    // 0 - Carry
    pub flags: u8,
    memory: [u8; MEMORY_SIZE as usize], // 16 bit address model. Going from $0000 to $FFFF
}

impl Default for CPU {
    fn default() -> Self {
        let memory: [u8; MEMORY_SIZE as usize] = [0; MEMORY_SIZE as usize];
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            flags: 0,
            memory: memory,
        }
    }
}

impl CPU {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) {
        let all_op_codes: &HashMap<u8, &'static Opcode> = &(*OPCODES_MAP);

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let current_program_counter_state = self.program_counter;

            let current_opcode = all_op_codes
                .get(&code)
                .unwrap_or_else(|| panic!("OP code {:X} not found", code));

            println!(
                "Current ops code: {:X?} and program counter {:X?} and CPU flags {:08b}",
                current_opcode, current_program_counter_state, self.flags
            );
            match code {
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&current_opcode.addressing_mode);
                }
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&current_opcode.addressing_mode);
                }
                0x0A | 0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&current_opcode.addressing_mode);
                }
                0x90 => self.bcc(&current_opcode.addressing_mode),
                0xB0 => self.bcs(&current_opcode.addressing_mode),
                0x50 => self.bvc(&current_opcode.addressing_mode),
                0x70 => self.bvs(&current_opcode.addressing_mode),
                0xF0 => self.beq(&current_opcode.addressing_mode),
                0xD0 => self.bne(&current_opcode.addressing_mode),
                0x10 => self.bpl(&current_opcode.addressing_mode),
                0x24 | 0x2C => {
                    self.bit(&current_opcode.addressing_mode);
                }
                0x30 => self.bmi(&current_opcode.addressing_mode),
                0x18 => self.clc(),
                0xD8 => self.cld(),
                0x58 => self.cli(),
                0xB8 => self.clv(),
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&current_opcode.addressing_mode);
                }
                0xE0 | 0xE4 | 0xEC => {
                    self.cpx(&current_opcode.addressing_mode);
                }
                0xCE | 0xDE | 0xC6 | 0xD6 => {
                    self.dec(&current_opcode.addressing_mode);
                }
                0xC0 | 0xC4 | 0xCC => {
                    self.cpy(&current_opcode.addressing_mode);
                }
                0xCA => self.dex(),
                0x88 => self.dey(),
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&current_opcode.addressing_mode);
                }
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    self.inc(&current_opcode.addressing_mode);
                }
                0xE8 => self.inx(),
                0xC8 => self.iny(),
                0x4C | 0x6C => self.jmp(&current_opcode.addressing_mode),
                0x20 => self.jsr(&current_opcode.addressing_mode),
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&current_opcode.addressing_mode);
                }
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&current_opcode.addressing_mode);
                }
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&current_opcode.addressing_mode);
                }
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(&current_opcode.addressing_mode);
                }
                0xEA => self.nop(),
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&current_opcode.addressing_mode);
                }
                0x48 => self.pha(),
                0x08 => self.php(),
                0x68 => self.pla(),
                0x28 => self.plp(),
                0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.rol(&current_opcode.addressing_mode),
                0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ror(&current_opcode.addressing_mode),
                0x40 => self.rti(),
                0x60 => self.rts(),
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&current_opcode.addressing_mode)
                }
                0x38 => self.sec(),
                0xF8 => self.sed(),
                0x78 => self.sei(),
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&current_opcode.addressing_mode);
                }
                0x86 | 0x96 | 0x8E => {
                    self.stx(&current_opcode.addressing_mode);
                }
                0x84 | 0x94 | 0x8C => {
                    self.sty(&current_opcode.addressing_mode);
                }
                0xAA => self.tax(),
                0xA8 => self.tay(),
                0x8A => self.txa(),
                0xBA => self.tsx(),
                0x9A => self.txs(),
                0x98 => self.tya(),
                0x00 => {
                    self.brk();
                    println!(
                        "Reached break with program counter: {:x}",
                        self.program_counter
                    );
                    return;
                }
                _ => return,
            }

            if current_program_counter_state == self.program_counter {
                //the addressing mode is a property of an instruction that defines how the CPU should interpret the next 1 or 2 bytes in the instruction stream.
                //Different addressing modes have different instruction sizes
                //There are no opcodes that occupy more than 3 bytes. CPU instruction size can be either 1, 2, or 3 bytes.
                self.program_counter += (current_opcode.bytes - 1) as u16;
            }
        }
    }

    // reset response for program state. Must be reset before program ROM actually run
    // 1. LOAD ROM
    // 2. RESET
    // 3. RUN
    pub fn reset(&mut self) {
        self.stack_pointer = STACK_STARTING_POINTER;
        self.accumulator = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.flags = 0b0000_0000;
        self.program_counter = self.mem_read_u16(RESET_INTERRUPT_ADDR);
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        self.memory[PROGRAM_ROM_MEMORY_ADDRESS_START as usize
            ..(PROGRAM_ROM_MEMORY_ADDRESS_START as usize + program.len())]
            .copy_from_slice(&program[..]);

        self.mem_write_u16(RESET_INTERRUPT_ADDR, PROGRAM_ROM_MEMORY_ADDRESS_START);
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_read_u16(&self, addr: u16) -> u16 {
        let lsb = self.mem_read(addr) as u16;
        let msb = self.mem_read(addr + 1) as u16;

        (msb << 8) | (lsb as u16)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
    }

    pub fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lsb = (data & 0xFF) as u8;
        let hsb = (data >> 8) as u8;

        self.mem_write(addr, lsb);
        self.mem_write(addr + 1, hsb);
    }

    pub fn get_indirect_lookup(&self, lookup_addr: u16) -> u16 {
        let lsb = self.mem_read(lookup_addr);
        let hsb = self.mem_read(lookup_addr.wrapping_add(1));
        (hsb as u16) << 8 | (lsb as u16)
    }

    pub fn update_negative_and_zero_flags(&mut self, result: u8) {
        self.update_zero_flag(result);
        self.update_negative_flag(result);
    }

    fn update_zero_flag(&mut self, result: u8) {
        if result == 0 {
            self.set_zero_flag();
        } else {
            self.clear_zero_flag();
        }
    }

    fn update_negative_flag(&mut self, result: u8) {
        if result & 0b1000_0000 != 0 {
            self.flags |= 0b1000_0000; //set negative flag to 1
        } else {
            self.flags &= 0b0111_1111; //set negative flag to 0
        }
    }

    fn update_carry_and_overflow_flag(&mut self, result: Option<u8>, op: MathematicalOperation) {
        match result {
            Some(_e) => {
                self.clear_overflow_flag();
                if op == MathematicalOperation::Sub {
                    self.set_carry_flag();
                } else {
                    self.clear_carry_flag();
                }
            }
            None => {
                self.clear_overflow_flag();
                if op == MathematicalOperation::Sub {
                    self.clear_carry_flag();
                } else {
                    self.set_carry_flag();
                }
            }
        }
    }

    //# SET CPU FLAG
    fn set_zero_flag(&mut self) {
        self.flags |= 0b0000_00010;
    }

    fn clear_zero_flag(&mut self) {
        self.flags &= 0b1111_1101;
    }

    fn set_break_flag(&mut self) {
        self.flags |= 0b0010_0000;
    }

    fn clear_break_flag(&mut self) {
        self.flags &= 0b1101_1111;
    }

    fn set_overflow_flag(&mut self) {
        self.flags |= 0b0100_0000;
    }

    fn clear_overflow_flag(&mut self) {
        self.flags &= 0b1011_1111;
    }

    fn set_carry_flag(&mut self) {
        self.flags |= 0b0000_0001;
    }

    fn clear_carry_flag(&mut self) {
        self.flags &= 0b1111_1110;
    }

    fn set_decimal_flag(&mut self) {
        self.flags |= 0b0000_1000;
    }

    fn clear_decimal_flag(&mut self) {
        self.flags &= 0b1111_0111;
    }

    fn set_interrupt_disable(&mut self) {
        self.flags |= 0b0010_0000;
    }

    fn clear_interrupt_disable(&mut self) {
        self.flags &= 0b1101_0000;
    }

    // Stack operator
    fn get_address_from_stack(&self) -> u16 {
        let absolute_stack_pointer = format!("01{:X}", self.stack_pointer);
        u16::from_str_radix(&absolute_stack_pointer, 16).unwrap()
    }

    // Notice one thing the 6502 stack address is starting from top to down.
    // For EG: if we got 2 value added to the stack, first one gonna be inserted
    // at $01FF and second one at $01FE so we add we actually decrement the address pointer by 1
    fn insert_address_into_stack(&mut self) -> u16 {
        let address = self.get_address_from_stack();
        self.stack_pointer -= 1;
        address
    }

    fn pop_address_from_stack(&mut self) -> u16 {
        self.stack_pointer += 1;
        self.get_address_from_stack()
    }

    fn pop_address_from_stack_u8(&mut self) -> u8 {
        self.stack_pointer += 1;
        self.mem_read((STACK_STARTING_POINTER as u16) + self.stack_pointer as u16)
    }

    fn get_operand_addr(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::AbsoluteX => {
                let pos = self.mem_read_u16(self.program_counter);
                pos.wrapping_add(self.register_x as u16) as u16
            }
            AddressingMode::AbsoluteY => {
                let pos = self.mem_read_u16(self.program_counter);
                pos.wrapping_add(self.register_y as u16) as u16
            }
            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);
                let lookup_addr = base.wrapping_add(self.register_x);

                // Indirect lookup
                self.get_indirect_lookup(lookup_addr as u16)
            }
            AddressingMode::IndirectY => {
                let lookup_addr = self.mem_read(self.program_counter);

                // Indirect Lookup
                let addr = self.get_indirect_lookup(lookup_addr as u16);

                addr.wrapping_add(self.register_y as u16)
            }
            AddressingMode::Relative => self.program_counter,
            AddressingMode::NoneAddressing => panic!("Mode not known"),
            _ => panic!("Mode not known"),
        }
    }

    //add_relative_displacement_to_program_counter because of little endianess of cpu 6502.
    // we must manually do add/sub on program counter.
    fn add_relative_displacement_to_program_counter(&mut self, step: u8) {
        let mut lsb_program_counter = (self.program_counter & 0xFF) as u8;
        let mut msb_program_counter = (self.program_counter >> 8) as u8;

        let step_as_i8 = step as i8;
        let step_as_i8_mult_minus_one = -step_as_i8;
        let positive = step_as_i8.signum() == 1;

        if positive {
            let wrap = lsb_program_counter.checked_add(step);
            lsb_program_counter = lsb_program_counter.wrapping_add(step);
            match wrap {
                Some(_) => (),
                None => msb_program_counter = msb_program_counter.wrapping_add(1),
            }
        } else {
            let wrap = lsb_program_counter.checked_sub(step_as_i8_mult_minus_one as u8);
            lsb_program_counter = lsb_program_counter.wrapping_sub(step_as_i8_mult_minus_one as u8);

            match wrap {
                Some(_) => (),
                None => msb_program_counter = msb_program_counter.wrapping_sub(1),
            }
        }

        self.program_counter = (msb_program_counter as u16) << 8 | (lsb_program_counter as u16);
    }

    // lda https://www.nesdev.org/obelisk-6502-guide/reference.html#LDA
    // Load Accumulator
    fn lda(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.accumulator = param;
        self.update_negative_and_zero_flags(self.accumulator);
    }

    // adc https://www.nesdev.org/obelisk-6502-guide/reference.html#ADC
    // Add with carry
    fn adc(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        let old_accumulator = self.accumulator;

        self.accumulator = self.accumulator.wrapping_add(param);

        self.update_negative_and_zero_flags(self.accumulator);
        self.update_carry_and_overflow_flag(
            old_accumulator.checked_add(param),
            MathematicalOperation::Add,
        );
    }

    // and https://www.nesdev.org/obelisk-6502-guide/reference.html#AND
    // Logical AND
    fn and(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.accumulator &= param;
        self.update_negative_and_zero_flags(self.accumulator);
    }

    // asl https://www.nesdev.org/obelisk-6502-guide/reference.html#ASL
    // ASL Arithmetic Shift Left
    fn asl(&mut self, addressing_mode: &AddressingMode) {
        let seventhBit = format!("{:08b}", self.accumulator)
            .chars()
            .collect::<Vec<char>>()[0];

        // If accumulator mode, we changing the value.
        // If not, we modifying shift left content of address read from that mode
        let mut result: u8;
        if *addressing_mode == AddressingMode::Accumulator {
            self.accumulator = self.accumulator << 1;
            result = self.accumulator
        } else {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);
            result = param << 1;
            self.accumulator = result;
            self.mem_write(operand_addr, result)
        }

        self.update_negative_and_zero_flags(result);
        // Update carry flags with old sventh bit
        let bit = format!("00000000{}", seventhBit);
        let new_bits = u8::from_str_radix(&bit, 2).unwrap();
        self.flags |= new_bits;
    }

    // bcc - Branch if Carry Clear
    // https://www.nesdev.org/obelisk-6502-guide/reference.html#BCC
    fn bcc(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        let step = param + 1;

        // if carry flag is not set
        if self.flags & 0x0000_00001 == 0 {
            self.add_relative_displacement_to_program_counter(step);
        }
    }

    // bcs Branch if Carry set
    fn bcs(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        let step = param + 1;

        // if carry flag is not set
        if self.flags & 0x0000_00001 == 1 {
            self.add_relative_displacement_to_program_counter(step);
        }
    }
    // beq Branch if Equal
    fn beq(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        let step = param + 1;

        if self.flags & 0x0000_0010 == 2 {
            // Zero flags is set
            self.add_relative_displacement_to_program_counter(step);
        }
    }
    // bit test
    fn bit(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);
    }
    // BMI Branch if Minus
    fn bmi(&mut self, addressing_mode: &AddressingMode) {
        // If negative flag is set
        if self.flags & 0b1000_0000 == 255 {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);

            let step = param + 1;
            self.add_relative_displacement_to_program_counter(step);
        }
    }

    // BNE Branch if not equal
    fn bne(&mut self, addressing_mode: &AddressingMode) {
        if self.flags & 0b0000_0010 == 0 {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);

            let step = param + 1;
            self.add_relative_displacement_to_program_counter(step);
        }
    }
    // BPL Branch if Positive
    fn bpl(&mut self, addressing_mode: &AddressingMode) {
        // If negative flag is clear
        if self.flags & 0b1000_0000 == 0 {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);

            let step = param + 1;
            self.add_relative_displacement_to_program_counter(step);
        }
    }

    // BRK Force Interrupt
    fn brk(&mut self) {
        // save program counter and
        let old_program_counter = self.program_counter;
        let lsb = (old_program_counter & 0xFF) as u8;
        let hsb = (old_program_counter >> 8) as u8;

        self.memory[self.insert_address_into_stack() as usize] = lsb;
        self.memory[self.insert_address_into_stack() as usize] = hsb;

        // update CPU status flags into the stack
        self.memory[self.insert_address_into_stack() as usize] = self.flags;

        // set IRQ interupt vector
        self.program_counter = self.mem_read_u16(RESET_INTERRUPT_ADDR);
        // set break flag to 1
        self.set_break_flag();
    }

    //bvc - Branch if Overflow Clear
    fn bvc(&mut self, addressing_mode: &AddressingMode) {
        // if overflow clear
        if self.flags & 0x0000_0000 == 0 {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);

            let jump_addr = self.program_counter.wrapping_add(param as u16);
            self.program_counter = jump_addr
        }
    }

    //bvs - Branch if Overflow Set
    fn bvs(&mut self, addressing_mode: &AddressingMode) {
        // if overflow is set
        if self.flags & 0b0100_0000 == 0b0100_0000 {
            let operand_addr = self.get_operand_addr(addressing_mode);
            let param = self.mem_read(operand_addr);

            let jump_addr = self.program_counter.wrapping_add(param as u16);
            self.program_counter = jump_addr
        }
    }
    //CLC - Clear Carry Flag
    fn clc(&mut self) {
        self.clear_carry_flag();
    }

    // CLD - Clear Decimal Mode
    fn cld(&mut self) {
        self.clear_decimal_flag();
    }

    //CLI - Clear Interrupt Disable
    fn cli(&mut self) {
        self.clear_interrupt_disable();
    }

    //CLV - Clear Overflow Flag
    fn clv(&mut self) {
        self.clear_overflow_flag();
    }

    //CMP - Compare
    fn cmp(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        if self.accumulator >= param {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        self.update_negative_and_zero_flags(self.accumulator.wrapping_sub(param))
    }

    //CPX - Compare X Register
    fn cpx(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        if self.register_x >= param {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        self.update_negative_and_zero_flags(self.register_x.wrapping_sub(param));
    }

    //CPY - Compare Y Register
    fn cpy(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        if self.register_y >= param {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        self.update_negative_and_zero_flags(self.register_y.wrapping_sub(param));
    }

    // DEC - Decrement Memory
    // Subtracts one from the value held at a specified memory location setting the zero and negative flags as appropriate.
    fn dec(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.mem_write(operand_addr, param.wrapping_sub(1));
        self.update_negative_and_zero_flags(param.wrapping_sub(1));
    }

    // DEX - Decrement X Register
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_negative_and_zero_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_negative_and_zero_flags(self.register_y);
    }

    // EOR Exclusive OR
    // An exclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn eor(&mut self, addressing_mode: &AddressingMode) {
        let addr = self.get_operand_addr(addressing_mode);
        let data = self.mem_read(addr);

        self.accumulator = self.accumulator ^ data
    }

    //INC - Increment Memory
    fn inc(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let mut data = self.mem_read(operand_addr);
        data = data.wrapping_add(1);
        self.mem_write(operand_addr, data);
        self.update_negative_and_zero_flags(data);
    }

    // INX - Increment X Register
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_negative_and_zero_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_negative_and_zero_flags(self.register_y);
    }

    // Jump
    fn jmp(&mut self, addressing_mode: &AddressingMode) {}

    // JSR - Jump to Subroutine
    //The JSR instruction pushes the address (minus one) of the return point on to the stack and then sets the program counter to the target memory address.
    fn jsr(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);

        let old_program_counter = self.program_counter + 1;
        let lsb = (old_program_counter & 0xFF) as u8;
        let hsb = (old_program_counter >> 8) as u8;
        self.memory[self.insert_address_into_stack() as usize] = hsb;
        self.memory[self.insert_address_into_stack() as usize] = lsb;

        // update program counter (to jump to a subroutine)
        self.program_counter = operand_addr;
    }

    //NOP - No Operation
    // The NOP instruction causes no changes to the processor other than the normal incrementing of the program counter to the next instruction.
    fn nop(&mut self) {
        //do nothing
    }

    // ORA - Logical Inclusive OR
    // An inclusive OR is performed, bit by bit, on the accumulator contents using the contents of a byte of memory.
    fn ora(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let mut data = self.mem_read(operand_addr);
        data = self.accumulator | data;
        self.accumulator = data;
        self.update_negative_and_zero_flags(data);
    }

    // PHA - Push Accumulator
    // Pushes a copy of the accumulator on to the stack.
    fn pha(&mut self) {
        let current_accumulator = self.accumulator;
        self.mem_write(self.get_address_from_stack(), current_accumulator);
        self.insert_address_into_stack();
    }
    //PHP - Push Processor Status
    //Pushes a copy of the status flags on to the stack.
    fn php(&mut self) {
        let current_flag = self.flags;
        self.mem_write(self.get_address_from_stack(), current_flag);
        self.insert_address_into_stack();
    }

    fn pla(&mut self) {
        let data_from_stack = self.pop_address_from_stack();
        let data_converted_8bit = (data_from_stack) as u8;
        self.accumulator = data_converted_8bit;
        self.update_negative_and_zero_flags(data_converted_8bit);
    }
    fn plp(&mut self) {
        let data_from_stack = self.pop_address_from_stack();
        let data_converted_8bit = (data_from_stack) as u8;
        self.flags = data_converted_8bit;
        self.update_negative_and_zero_flags(data_converted_8bit);
    }

    fn rol(&mut self, addressing_mode: &AddressingMode) {
        let carry = format!("{:08b}", self.flags).chars().collect::<Vec<char>>()[7];

        let old_param: u8;
        let result: u8;
        if *addressing_mode == AddressingMode::Accumulator {
            old_param = self.accumulator;
            self.accumulator = (self.accumulator << 1) | carry as u8;
            result = self.accumulator;
        } else {
            let operand_addr = self.get_operand_addr(addressing_mode);
            old_param = self.mem_read(operand_addr);
            self.mem_write(operand_addr, (old_param << 1) | carry as u8);
            result = old_param << 1;
        }

        //Bit 0 is filled with the current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
        if old_param >> 7 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        self.update_negative_and_zero_flags(result);
    }

    fn ror(&mut self, addressing_mode: &AddressingMode) {
        let carry = format!("{:08b}", self.flags).chars().collect::<Vec<char>>()[7];

        let old_param: u8;
        let result: u8;
        //Bit 7 is filled with the current value of the carry flag
        if *addressing_mode == AddressingMode::Accumulator {
            old_param = self.accumulator;
            self.accumulator = (self.accumulator >> 1) | ((carry as u8) << 7);
            result = self.accumulator;
        } else {
            let operand_addr = self.get_operand_addr(addressing_mode);
            old_param = self.mem_read(operand_addr);
            self.mem_write(operand_addr, (old_param >> 1) | ((carry as u8) << 7));
            result = old_param << 1;
        }

        //Bit 0 is filled with the current value of the carry flag whilst the old bit 7 becomes the new carry flag value.
        if old_param & 1 == 1 {
            self.set_carry_flag();
        } else {
            self.clear_carry_flag();
        }

        self.update_negative_and_zero_flags(result);
    }

    fn rti(&mut self) {
        let stack_addr = self.pop_address_from_stack_u8();
        self.flags = stack_addr;
        self.program_counter = self.pop_address_from_stack();
    }

    fn rts(&mut self) {
        let lsb = self.memory[self.pop_address_from_stack() as usize];
        let hsb = self.memory[self.pop_address_from_stack() as usize];

        let new_program_counter = (hsb as u16) << 8 | (lsb as u16);
        self.program_counter = new_program_counter + 1;
    }

    // ldx https://www.nesdev.org/obelisk-6502-guide/reference.html#LDX
    // Load X Register
    fn ldx(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.register_x = param;
        self.update_negative_and_zero_flags(self.register_x);
    }

    fn ldy(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.register_y = param;
        self.update_negative_and_zero_flags(self.register_y);
    }

    // LSR - Logical Shift Right
    fn lsr(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);
    }

    fn sta(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        self.mem_write(operand_addr, self.accumulator);
    }

    // SBC - Subtract with Carry
    // This instruction subtracts the contents of a memory location to the accumulator together with the not of the carry bit. If overflow occurs the carry bit is clear, this enables multiple byte subtraction to be performed.
    fn sbc(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        let old_accumulator = self.accumulator;

        self.accumulator = self.accumulator.wrapping_sub(param);
        self.update_negative_and_zero_flags(self.accumulator);
        self.update_carry_and_overflow_flag(
            old_accumulator.checked_sub(param),
            MathematicalOperation::Sub,
        );
    }

    //SEC - Set Carry Flag
    // Set the carry flag to one.
    fn sec(&mut self) {
        self.set_carry_flag();
    }
    //fn set(&mut self, addressing_mode: &AddressingMode) {}
    // SED - Set Decimal Flag
    //
    fn sed(&mut self) {
        self.set_decimal_flag();
    }
    fn sei(&mut self) {
        self.set_interrupt_disable();
    }

    fn stx(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        self.mem_write(operand_addr, self.register_x);
    }

    fn sty(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        self.mem_write(operand_addr, self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.accumulator;
        self.update_negative_and_zero_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.accumulator = self.register_x;
        self.update_negative_and_zero_flags(self.accumulator);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_negative_and_zero_flags(self.register_x);
    }
    fn tay(&mut self) {
        self.register_y = self.accumulator;
        self.update_negative_and_zero_flags(self.register_x);
    }
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }
    fn tya(&mut self) {
        self.accumulator = self.register_y;
        self.update_negative_and_zero_flags(self.accumulator);
    }
}
