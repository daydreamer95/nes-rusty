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

    pub fn load_program(&mut self, program: Vec<u8>) {
        self.memory[PROGRAM_ROM_MEMORY_ADDRESS_START as usize
            ..(PROGRAM_ROM_MEMORY_ADDRESS_START as usize + program.len())]
            .copy_from_slice(&program[..]);
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_read_u16(&self, addr: u16) -> u16 {
        let lsb = self.mem_read(addr) as u16;
        let msb = self.mem_read(addr + 1) as u16;

        println!("result: {:#06x}", (msb << 8) | (lsb as u16) as u16);
        (msb << 8) | (lsb as u16)
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data
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

    //# SET CPU FLAG
    fn set_zero_flag(&mut self) {
        self.flags |= 0b0000_00010;
    }

    fn clear_zero_flag(&mut self) {
        self.flags &= 0b1111_1101;
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

    // lda https://www.nesdev.org/obelisk-6502-guide/reference.html#LDA
    // Load Accumulator
    fn lda(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        let param = self.mem_read(operand_addr);

        self.accumulator = param;
        self.update_negative_and_zero_flags(self.accumulator);
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

    fn sta(&mut self, addressing_mode: &AddressingMode) {
        let operand_addr = self.get_operand_addr(addressing_mode);
        self.mem_write(operand_addr, self.accumulator);
    }
}
