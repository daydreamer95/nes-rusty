use crate::{cartridge, opharn::Orphan};
use bitflags::bitflags;

pub trait Context: Sized {
    fn state_mut(&mut self) -> &mut PPU;
    fn state(&self) -> &PPU;

    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);

    fn peek_video_memory(&self, _address: u16) -> u8;
    fn poke_video_memory(&mut self, _address: u16, _value: u8);
}

pub trait Interface: Sized + Context {
    // fn mem_write(&mut self, addr: u16, data: u8) {
    //     self.state_mut().addr.update(data);
    // }

    fn write_to_ctrl(&mut self, data: u8) {
        println!("ppu write_to_ctrl");
        let before_nmi_status = self.state_mut().ctrl.generate_vblank_nmi();
        self.state_mut().ctrl.update(data);
        if !before_nmi_status
            && self.state_mut().ctrl.generate_vblank_nmi()
            && self.state().status.is_in_vblank()
        {
            println!("ppu nmi_interrupt");
            self.state_mut().nmi_interrupt = Some(1);
        }
    }

    fn increment_vram_addr(&mut self) {
        println!("ppu increment_vram_addr");
        let addr_increment = self.state().ctrl.vram_addr_increment();
        self.state_mut().addr.increment(addr_increment);
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        println!("ppu mem_read");
        let addr = self.state().addr.get();
        self.increment_vram_addr();

        match addr {
            // Pattern Tables (CHR ROMS)
            0..=0x1fff => {
                let result = self.state().internal_data_buf;
                self.state_mut().internal_data_buf = self.state().chr_rom[addr as usize];
                result
            }
            // Name Tables ( VRAMS) or we can call screen state
            // 4 KiB of addressable space. Two "additional" screens have to be mapped to existing ones.
            // The way they are mapped depends on the mirroring type, specified by a game (iNES files have this info in the header)
            0x2000..=0x2fff => {
                let result = self.state().internal_data_buf;
                self.state_mut().internal_data_buf =
                    self.state().vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            // Palettes
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                addr
            ),
            //0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        // println!("ppu poll_nmi_interrupt");
        self.state_mut().nmi_interrupt.take()
    }

    fn tick(&mut self, cycles: u8) -> bool {
        self.state_mut().cycles += cycles as usize;
        if self.state().cycles >= 341 {
            self.state_mut().cycles -= 341;
            self.state_mut().scanline += 1;

            if self.state().scanline == 241 {
                self.state_mut().status.set_vblank_status(true);
                self.state_mut().status.set_sprite_zero_hit(true);
                if self.state_mut().ctrl.generate_vblank_nmi() {
                    println!("ppu interrupt: {:?}", self.state().cycles);
                    self.state_mut().nmi_interrupt = Some(1);
                }
            }

            if self.state().scanline >= 262 {
                println!("trigger nmi");
                self.state_mut().scanline = 0;
                self.state_mut().nmi_interrupt = None;
                self.state_mut().status.set_sprite_zero_hit(true);
                self.state_mut().status.reset_vblank_status();
                self.state_mut().frame_completed = true;
                return true;
            }
        }

        return false;
    }

    fn write_to_mask(&mut self, value: u8) {
        println!("ppu write_to_mask");
        self.state_mut().mask.update(value);
    }

    fn read_status(&mut self) -> u8 {
        println!("ppu read_status");
        let data = self.state().status.snapshot();
        self.state_mut().status.reset_vblank_status();
        self.state_mut().addr.reset_latch();
        self.state_mut().scroll.reset_latch();
        data
    }

    fn write_to_oam_addr(&mut self, value: u8) {
        println!("ppu write_to_oam_addr");
        self.state_mut().oam_addr = value;
    }

    fn write_to_oam_data(&mut self, value: u8) {
        println!("ppu write_to_oam_data");
        let oam_addr = self.state().oam_addr;
        self.state_mut().oam_data[oam_addr as usize] = value;
        self.state_mut().oam_addr = oam_addr.wrapping_add(1);
    }

    fn read_oam_data(&self) -> u8 {
        println!("ppu read_oam_data");
        let oam_addr = self.state().oam_addr;
        self.state().oam_data[oam_addr as usize]
    }

    fn write_to_scroll(&mut self, value: u8) {
        println!("ppu write_to_scroll");
        self.state_mut().scroll.write(value);
    }

    fn write_to_ppu_addr(&mut self, value: u8) {
        println!("ppu write_to_ppu_addr");
        self.state_mut().addr.update(value);
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        println!("ppu mem_write");
        let addr = self.state().addr.get();
        match addr {
            0..=0x1fff => println!("attempt to write to chr rom space {}", addr),
            0x2000..=0x2fff => {
                let mirror_vram_addr = self.mirror_vram_addr(addr);
                self.state_mut().vram[mirror_vram_addr as usize] = data;
            }
            0x3000..=0x3eff => unimplemented!("addr {} shouldn't be used in reallity", addr),

            //Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.state_mut().palette_table[(add_mirror - 0x3f00) as usize] = data;
            }
            0x3f00..=0x3fff => {
                self.state_mut().palette_table[(addr - 0x3f00) as usize] = data;
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
        self.increment_vram_addr();
    }

    fn read_data(&mut self) -> u8 {
        println!("ppu read_data");
        let addr = self.state().addr.get();

        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                let result = self.state().internal_data_buf;
                self.state_mut().internal_data_buf = self.state().chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.state().internal_data_buf;
                self.state_mut().internal_data_buf =
                    self.state().vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => unimplemented!("addr {} shouldn't be used in reallity", addr),

            //Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.state_mut().palette_table[(add_mirror - 0x3f00) as usize]
            }

            0x3f00..=0x3fff => self.state().palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    fn write_oam_dma(&mut self, data: &[u8; 256]) {
        println!("ppu write_oam_dma");
        for x in data.iter() {
            let oarm_addr = self.state().oam_addr;
            self.state_mut().oam_data[oarm_addr as usize] = *x;
            self.state_mut().oam_addr = self.state().oam_addr.wrapping_add(1);
        }
    }
}

impl<T: Context> Interface for T {}
impl<T: Context> Private for T {}

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

    // Cycles
    cycles: usize,
    scanline: u16,
    pub nmi_interrupt: Option<u8>,

    //
    pub frame_completed: bool,

    //Registers: Used by CPU for communication
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub oam_addr: u8,
    pub scroll: ScrollRegister,
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
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            oam_addr: 0,
            oam_data: [0; 256],
            scroll: ScrollRegister::new(),
            addr: AddrRegister::new(),
            data: 0,
            oam_dma: 0,
            chr_rom: chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            mirroring: mirroring,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
            frame_completed: true,
        }
    }

    // https://wiki.nesdev.org/w/index.php/Mirroring
    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]
    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
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
}

trait Private: Sized + Context {
    fn increment_vram_addr(&mut self) {
        let vram_address = self.state_mut().ctrl.vram_addr_increment();
        self.state_mut().addr.increment(vram_address);
    }

    // https://wiki.nesdev.org/w/index.php/Mirroring
    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]
    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.state().mirroring, name_table) {
            (cartridge::Mirroring::Vertical, 2) | (cartridge::Mirroring::Vertical, 3) => {
                vram_index - 0x800
            }
            (cartridge::Mirroring::Horizontal, 2) => vram_index - 0x400,
            (cartridge::Mirroring::Horizontal, 1) => vram_index - 0x400,
            (cartridge::Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
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

    // flipping boolean each time so we know we interact with high pointer or low pointer
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

    fn generate_vblank_nmi(&mut self) -> bool {
        return self.contains(ControlRegister::GENERATE_NMI);
    }

    pub fn nametable_addr(&self) -> u16 {
        match self.bits & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => panic!("not possible"),
        }
    }

    pub fn sprt_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn bknd_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKROUND_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn sprite_size(&self) -> u8 {
        if !self.contains(ControlRegister::SPRITE_SIZE) {
            8
        } else {
            16
        }
    }

    pub fn master_slave_select(&self) -> u8 {
        if !self.contains(ControlRegister::SPRITE_SIZE) {
            0
        } else {
            1
        }
    }
}

bitflags! {

    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red
    // |+-------- Emphasize green
    // +--------- Emphasize blue
    pub struct MaskRegister: u8 {
        const GREYSCALE               = 0b00000001;
        const LEFTMOST_8PXL_BACKGROUND  = 0b00000010;
        const LEFTMOST_8PXL_SPRITE      = 0b00000100;
        const SHOW_BACKGROUND         = 0b00001000;
        const SHOW_SPRITES            = 0b00010000;
        const EMPHASISE_RED           = 0b00100000;
        const EMPHASISE_GREEN         = 0b01000000;
        const EMPHASISE_BLUE          = 0b10000000;
    }
}

pub enum Color {
    Red,
    Green,
    Blue,
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0b00000000)
    }

    pub fn is_grayscale(&self) -> bool {
        self.contains(MaskRegister::GREYSCALE)
    }

    pub fn leftmost_8pxl_background(&self) -> bool {
        self.contains(MaskRegister::LEFTMOST_8PXL_BACKGROUND)
    }

    pub fn leftmost_8pxl_sprite(&self) -> bool {
        self.contains(MaskRegister::LEFTMOST_8PXL_SPRITE)
    }

    pub fn show_background(&self) -> bool {
        self.contains(MaskRegister::SHOW_BACKGROUND)
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SHOW_SPRITES)
    }

    pub fn emphasise(&self) -> Vec<Color> {
        let mut result = Vec::<Color>::new();
        if self.contains(MaskRegister::EMPHASISE_RED) {
            result.push(Color::Red);
        }
        if self.contains(MaskRegister::EMPHASISE_BLUE) {
            result.push(Color::Blue);
        }
        if self.contains(MaskRegister::EMPHASISE_GREEN) {
            result.push(Color::Green);
        }

        result
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}

pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub latch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            latch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if !self.latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.latch = !self.latch;
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }
}

bitflags! {

    // 7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- Least significant bits previously written into a PPU register
    // |||        (due to register not being updated for this address)
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    pub struct StatusRegister: u8 {
        const NOTUSED          = 0b00000001;
        const NOTUSED2         = 0b00000010;
        const NOTUSED3         = 0b00000100;
        const NOTUSED4         = 0b00001000;
        const NOTUSED5         = 0b00010000;
        const SPRITE_OVERFLOW  = 0b00100000;
        const SPRITE_ZERO_HIT  = 0b01000000;
        const VBLANK_STARTED   = 0b10000000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b00000000)
    }

    pub fn set_vblank_status(&mut self, status: bool) {
        self.set(StatusRegister::VBLANK_STARTED, status);
    }

    pub fn set_sprite_zero_hit(&mut self, status: bool) {
        self.set(StatusRegister::SPRITE_ZERO_HIT, status);
    }

    pub fn set_sprite_overflow(&mut self, status: bool) {
        self.set(StatusRegister::SPRITE_OVERFLOW, status);
    }

    pub fn reset_vblank_status(&mut self) {
        self.remove(StatusRegister::VBLANK_STARTED);
    }

    pub fn is_in_vblank(&self) -> bool {
        self.contains(StatusRegister::VBLANK_STARTED)
    }

    pub fn snapshot(&self) -> u8 {
        self.bits
    }
}
