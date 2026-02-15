use crate::cartridge;
use bitflags::bitflags;

//https://www.nesdev.org/wiki/PPU_registers
pub struct PPU {
    internal_data_buf: u8,

    // Charater ROM and mirroring from catridge
    pub chr_rom: Vec<u8>,
    pub mirroring: cartridge::Mirroring,
    //Palaette tables
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],

    palette_ram: [u8; 32],
    sprite_list_ram: [u8; 256],
    secondary_sprite_list_ram: [u8; 32],

    //Registers
    pub ctrl: ControlRegister,
    pub mask: u8,
    pub status: u8,
    pub oam_addr: u8,
    pub scroll: u8,
    pub addr: AddrRegister,
    pub data: u8,
    pub oam_dma: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: cartridge::Mirroring) -> PPU {
        PPU {
            internal_data_buf: 0,
            palette_ram: [0; 32],
            sprite_list_ram: [0; 256],
            secondary_sprite_list_ram: [0; 32],
            ctrl: ControlRegister::new(),
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: [0; 256],
            scroll: 0,
            addr: AddrRegister::new(),
            data: 0,
            oam_dma: 0,
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            mirroring: mirroring,
        }
    }

    fn write_to_ppu_addr(&mut self, data: u8) {
        self.addr.update(data);
    }

    fn write_to_ctrl(&mut self, data: u8) {
        self.ctrl.update(data);
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    // https://wiki.nesdev.org/w/index.php/Mirroring
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.mirroring, name_table) {
            (cartridge::Mirroring::Vertical, 2) | (cartridge::Mirroring::Vertical, 3) => {
                vram_index - 0x800
            }
            (cartridge::Mirroring::Horizontal, 2) => vram_index - 0x400,
            (cartridge::Mirroring::Horizontal, 1) => vram_index - 0x400,
            (cartridge::Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                addr
            ),
            //0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }
}

//https://www.nesdev.org/wiki/PPU_registers#PPUADDR_-_VRAM_address_($2006_write)
pub struct AddrRegister {
    value: (u8, u8),
    high_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> AddrRegister {
        AddrRegister {
            value: (0, 0), // high byte first, lo byte second
            high_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.high_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.get() > 0x3fff {
            //mirror down addr above 0x3fff
            self.set(self.get() & 0b11111111111111);
        }
        self.high_ptr = !self.high_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111); //mirror down addr above 0x3fff
        }
    }

    pub fn reset_latch(&mut self) {
        self.high_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

bitflags! {
   // 7  bit  0
   // ---- ----
   // VPHB SINN
   // |||| ||||
   // |||| ||++- Base nametable address
   // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
   // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
   // |||| |     (0: add 1, going across; 1: add 32, going down)
   // |||| +---- Sprite pattern table address for 8x8 sprites
   // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
   // |||+------ Background pattern table address (0: $0000; 1: $1000)
   // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
   // |+-------- PPU master/slave select
   // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
   // +--------- Generate an NMI at the start of the
   //            vertical blanking interval (0: off; 1: on)
   pub struct ControlRegister: u8 {
       const NAMETABLE1              = 0b00000001;
       const NAMETABLE2              = 0b00000010;
       const VRAM_ADD_INCREMENT      = 0b00000100;
       const SPRITE_PATTERN_ADDR     = 0b00001000;
       const BACKROUND_PATTERN_ADDR  = 0b00010000;
       const SPRITE_SIZE             = 0b00100000;
       const MASTER_SLAVE_SELECT     = 0b01000000;
       const GENERATE_NMI            = 0b10000000;
   }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b00000000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}
