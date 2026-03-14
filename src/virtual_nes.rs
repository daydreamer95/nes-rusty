use crate::cartridge::NesRom;
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
            nes_rom: nes_rom,
        };
        emulator
    }

    pub fn new(file_path: String) -> Emulator {
        let rom_contents = Emulator::load_rom(file_path);

        let nes_rom = NesRom::new(&rom_contents).unwrap();
        Emulator {
            cpu_vram: [0; 2048],
            cpu_state: cpu::CPU::new(),
            ppu_state: ppu::PPU::new(nes_rom.chr_rom.clone(), nes_rom.mirror.clone()),
            nes_rom: nes_rom,
        }
    }

    pub fn load_rom(file_path: String) -> Vec<u8> {
        let contents = fs::read(file_path).expect("Should be able to read file and content");
        contents
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

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

impl Mem for Emulator {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet")
            }
            0x8000..=0xFFFF => {
                println!("UnImplement Rom mem access at {}", addr);
                0
            }
            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet");
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
