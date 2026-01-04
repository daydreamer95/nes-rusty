use crate::cartridge::NesRom;
use mos6502::cpu;
use std::fs;

pub struct Emulator {
    pub cpu_state: cpu::CPU,
    pub nes_rom: NesRom,
}

impl Emulator {
    pub fn new_with_gamecodes(game_codes: Vec<u8>) -> Emulator {
        let nes_rom_result = NesRom::new(&game_codes);
        let nes_rom = match nes_rom_result {
            Ok(rom_bytes) => rom_bytes,
            Err(error) => panic!("Failed to load nes game code with {:?}", error),
        };
        Emulator {
            cpu_state: cpu::CPU::new(),
            nes_rom: nes_rom,
        }
    }

    pub fn new(&mut self, file_path: String) -> Emulator {
        let rom_contents = self.load_rom(file_path);
        let nes_rom = NesRom::new(&rom_contents).unwrap();
        Emulator {
            cpu_state: cpu::CPU::new(),
            nes_rom: nes_rom,
        }
    }

    fn load_rom(&mut self, file_path: String) -> Vec<u8> {
        let contents = fs::read(file_path).expect("Should be able to read file and content");
        contents
    }
}
