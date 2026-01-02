#[derive(Debug, PartialEq, Eq)]
pub struct Cartridge {
    pub prg_rom: Vec<u8>, // Program-ROM
    pub chr_rom: Vec<u8>, // Character ROM ( Sprites)
    pub s_ram: Vec<u8>,   // Save RAM
    pub mapper: u8,       // mapper type
    pub mirror: u8,       // mirroring mode type
    pub battery: u8,      //battery present
}
