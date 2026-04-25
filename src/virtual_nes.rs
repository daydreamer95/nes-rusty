use crate::cartridge::NesRom;
use crate::opharn::Orphan;
use crate::ppu;
use mos6502::cpu;
use std::fs;

// TODO: this is stupid, might need to refactor
pub struct Emulator {
    cpu_vram: [u8; 2048],
    pub cpu_state: cpu::CPU,
    pub nes_rom: NesRom,
    pub ppu_state: ppu::PPU,
}

impl Emulator {
    pub fn new_with_gamecodes(game_codes: Vec<u8>) -> Emulator {
        let nes_rom_result = NesRom::new(&game_codes);
        let nes_rom = match nes_rom_result {
            Ok(rom_bytes) => rom_bytes,
            Err(error) => panic!("Failed to load nes game code with {:?}", error),
        };
        let emulator = Emulator {
            cpu_vram: [0; 2048],
            cpu_state: cpu::CPU::new(),
            ppu_state: ppu::PPU::new(nes_rom.chr_rom.clone(), nes_rom.mirror.clone()),
            nes_rom: nes_rom.clone(),
        };
        emulator
    }

    pub fn new(file_path: String) -> Emulator {
        let rom_contents = Emulator::load_rom(file_path);

        let nes_rom = NesRom::new(&rom_contents).unwrap();
        let emulator = Emulator {
            cpu_vram: [0; 2048],
            cpu_state: cpu::CPU::new(),
            ppu_state: ppu::PPU::new(nes_rom.chr_rom.clone(), nes_rom.mirror.clone()),
            nes_rom: nes_rom.clone(),
        };
        println!(
            "debug im start to write program rom with len {}",
            nes_rom.prg_rom.len()
        );
        println!("debug im done to write program rom");
        emulator
    }

    pub fn load_rom(file_path: String) -> Vec<u8> {
        let contents = fs::read(file_path).expect("Should be able to read file and content");
        contents
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.nes_rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            //mirror if needed
            addr = addr % 0x4000;
        }
        self.nes_rom.prg_rom[addr as usize]
    }

    // pub fn run_with_callback<F>(&mut self, mut callback: F)
    // where
    //     F: FnMut(&mut Emulator),
    // {
    //     loop {
    //         //let code = self.mem_read(self.cpu_state.program_counter);
    //         cpu::Interface::run_with_callback(self.new(), callback);
    //         //self.cpu_state.run();
    //         //callback(self);
    //     }
    // }
}

impl Context for Emulator {
    fn state_mut(&mut self) -> &mut Emulator {
        self
    }
    fn state(&self) -> &Emulator {
        self
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

trait Context: Sized {
    fn state_mut(&mut self) -> &mut Emulator;
    fn state(&self) -> &Emulator;

    //fn mem_read(&self, addr: u16) -> u8 {}
    //fn mem_write(&mut self, addr: u16, data: u8) {}

    fn mem_read(&self, addr: u16) -> u8 {
        self.state().cpu_vram[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.state_mut().cpu_vram[addr as usize] = data
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lsb = self.mem_read(addr) as u16;
        let msb = self.mem_read(addr + 1) as u16;

        (msb << 8) | (lsb as u16)
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lsb = (data & 0xFF) as u8;
        let hsb = (data >> 8) as u8;

        self.mem_write(addr, lsb);
        self.mem_write(addr + 1, hsb);
    }
}

impl<T: Context> Private for T {}
impl<T: Context> Interface for T {}

pub trait Private: Sized + Context {
    fn mem_read(&self, addr: u16) -> u8 {
        println!("mem_read good way {:X}", addr);
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.state().cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                return 0;
                //self.state().ppu_state.mem_read(_mirror_down_addr)
            }
            0x8000..=0xFFFF => self.state().read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        println!("write address {:X}", addr);
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.state_mut().cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
                // 0
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                //self.state_mut().ppu_state.mem_write(_mirror_down_addr, data);
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }

    fn run_with_callback(&mut self) {
        cpu::Interface::run_with_callback(self.newtype_mut());
    }
}

pub trait Interface: Sized + Context {
    fn newtype(&self) -> &Orphan<Self> {
        Orphan::<Self>::cast(self)
    }

    fn newtype_mut(&mut self) -> &mut Orphan<Self> {
        Orphan::<Self>::cast_mut(self)
    }

    fn run(&mut self) {
        loop {
            //let code = self.mem_read(self.cpu_state.program_counter);
            cpu::Interface::run(self.newtype_mut());
            //self.cpu_state.run();
            //callback(self);
        }
    }

    fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Emulator),
    {
        loop {
            //let code = self.mem_read(self.cpu_state.program_counter);
            Private::run_with_callback(self.state_mut());
            //self.cpu_state.run();
            callback(&mut self.state_mut());
        }
    }

    // reset response for program state. Must be reset before program ROM actually run
    // 1. LOAD ROM
    // 2. RESET
    // 3. RUN
    fn reset(&mut self) {
        cpu::Interface::reset(self.newtype_mut());
        //self.cpu_state.reset();
    }
}

impl<C: Context> cpu::Context for Orphan<C> {
    #[inline]
    fn state_mut(&mut self) -> &mut cpu::CPU {
        &mut self.as_mut().state_mut().cpu_state
    }

    #[inline]
    fn state(&self) -> &cpu::CPU {
        &self.as_ref().state().cpu_state
    }

    fn mem_read(&self, addr: u16) -> u8 {
        Private::mem_read(self.as_ref(), addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        Private::mem_write(self.as_mut(), addr, data)
    }
}
