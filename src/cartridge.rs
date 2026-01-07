#[derive(Debug, PartialEq)]
pub struct NesRom {
    pub prg_rom: Vec<u8>, // Program-ROM
    pub chr_rom: Vec<u8>, // Character ROM ( Sprites)
    //pub s_ram: Vec<u8>,   // Save RAM
    pub mapper: u8, // mapper type
    pub mirror: Mirroring, // mirroring mode type
                    //pub battery: u8,      //battery present
}

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

impl NesRom {
    //https://www.nesdev.org/wiki/INES
    pub fn new(raw: &Vec<u8>) -> Result<NesRom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        // Check flags 7 ( This is bytes 7 of ROM)
        // 76543210
        // ||||||||
        // |||||||+- VS Unisystem
        // ||||||+-- PlayChoice-10 (8 KB of Hint Screen data stored after CHR data)
        // ||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
        // ++++----- Upper nybble of mapper number
        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        // Flags 6 ( This is bytes 6 of ROM )
        // 76543210
        // ||||||||
        // |||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
        // |||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
        // ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
        // |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
        // ||||+---- 1: Alternative nametable layout
        // ++++----- Lower nybble of mapper number
        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(NesRom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            mirror: screen_mirroring,
        })
    }
}
