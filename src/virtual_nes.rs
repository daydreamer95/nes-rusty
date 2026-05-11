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
            // cycles: 0,
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
            // cycles: 0,
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

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.state().cpu_vram[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.state_mut().cpu_vram[addr as usize] = data
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
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

    fn tick(&mut self, cycles: u8) {
        // self.state_mut().cycles += cycles as usize;
        cpu::Context::tick(self.newtype_mut(), cycles);
        ppu::Interface::tick(self.newtype_mut(), cycles * 3);
    }
}

impl<T: Context> Private for T {}
impl<T: Context> Interface for T {}

pub trait Private: Sized + Context {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.state().cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => ppu::Interface::read_status(self.newtype_mut()),
            0x2004 => ppu::Interface::read_oam_data(self.newtype_mut()),
            0x2007 => ppu::Interface::read_data(self.newtype_mut()),
            0x4000..=0x4015 => {
                //ignore APU
                0
            }
            0x4016 => {
                // ignore joypad 1;
                0
            }

            0x4017 => {
                // ignore joypad 2
                0
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                println!("PPU read");
                let mirror_down_addr = addr & 0b00100000_00000111;
                Private::mem_read(self, mirror_down_addr)
                // ppu::Interface::mem_read(self.newtype_mut(), mirror_down_addr)
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                // return 0;
                // let _mirror_down_addr = addr & 0b00100000_00000111;
                // self.state_mut().mem_read(_mirror_down_addr)
            }
            0x8000..=0xFFFF => self.state().read_prg_rom(addr),
            _ => {
                println!("Ignoring mem access at {:x}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.state_mut().cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                ppu::Interface::write_to_ctrl(self.newtype_mut(), data);
            }
            0x2001 => {
                ppu::Interface::write_to_mask(self.newtype_mut(), data);
            }
            0x2002 => panic!("attempt to write to PPU status register"),
            0x2003 => {
                ppu::Interface::write_to_oam_addr(self.newtype_mut(), data);
            }
            0x2004 => {
                ppu::Interface::write_to_oam_data(self.newtype_mut(), data);
            }
            0x2005 => {
                ppu::Interface::write_to_scroll(self.newtype_mut(), data);
            }

            0x2006 => {
                ppu::Interface::write_to_ppu_addr(self.newtype_mut(), data);
            }
            0x2007 => {
                ppu::Interface::mem_write(self.newtype_mut(), addr, data);
            }
            0x4000..=0x4013 | 0x4015 => {
                //ignore APU
            }

            0x4016 => {
                // ignore joypad 1;
            }

            0x4017 => {
                // ignore joypad 2
            }

            // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = Private::mem_read(self, hi + i);
                }

                ppu::Interface::write_oam_dma(self.newtype_mut(), &buffer);

                // todo: handle this eventually
                // let add_cycles: u16 = if self.cycles % 2 == 1 { 514 } else { 513 };
                // self.tuck(add_cycles); //todo this will cause weird effects as PPU will have 513/514 * 3 ticks
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                println!("PPU write");
                let _mirror_down_addr = addr & 0b00100000_00000111;
                Private::mem_write(self, _mirror_down_addr, data);
                // ppu::Interface::mem_write(self.newtype_mut(), _mirror_down_addr, data);
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                // let _mirror_down_addr = addr & 0b00100000_00000111;
                // self.state_mut()
                //     .ppu_state
                //     .mem_write(_mirror_down_addr, data);

                // ppu::Interface::mem_write(self.state_mut(), _mirror_down_addr)
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }

    // fn tick(&mut self, cycles: u8) {
    //     // self.state_mut().cycles += cycles as usize;
    //     // cpu::Context::tick(self.newtype_mut(), cycles);
    //     ppu::Interface::tick(self.newtype_mut(), cycles * 3);
    // }
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
            cpu::Interface::run(self.newtype_mut());
        }
    }

    fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Emulator),
    {
        cpu::Interface::run_with_callback(
            self.newtype_mut(),
            move |orphan| {
                callback(orphan.as_mut().state_mut());
            },
            move |orphan| ppu::Interface::poll_nmi_interrupt(orphan.as_mut().newtype_mut()),
            |orphan, cycles| orphan.as_mut().tick(cycles),
        );
    }

    // reset response for program state. Must be reset before program ROM actually run
    // 1. LOAD ROM
    // 2. RESET
    // 3. RUN
    fn reset(&mut self) {
        cpu::Interface::reset(self.newtype_mut());
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

    fn mem_read(&mut self, addr: u16) -> u8 {
        Private::mem_read(self.as_mut(), addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        Private::mem_write(self.as_mut(), addr, data)
    }
}

impl<C: Context> ppu::Context for Orphan<C> {
    #[inline]
    fn state_mut(&mut self) -> &mut ppu::PPU {
        &mut self.as_mut().state_mut().ppu_state
    }

    #[inline]
    fn state(&self) -> &ppu::PPU {
        &self.as_ref().state().ppu_state
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        Private::mem_read(self.as_mut(), addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        Private::mem_write(self.as_mut(), addr, data)
    }

    fn peek_video_memory(&self, _address: u16) -> u8 {
        return 0;
    }

    fn poke_video_memory(&mut self, _address: u16, _value: u8) {}
}
