// src/main.rs
use byteorder::{BigEndian, ReadBytesExt};
use bytes::Buf;
use serde::Serialize;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::mem;
use std::slice;
use std::fs;
use std::{
    fs::File,
    io::{self, Read},
};
use std::str;
use bytes::Bytes;

static PREFIX : u8 = 0xCB; // use the prefixed opcodes for interpretation
static ENTRYPOINT_OFFSET : u64 = 0x102;
static HEADER_START : u64 = 0x134; // offset for cartridge metadata
//static HEADER_END : u64 = 0x14F;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub mnemonic: String,
    pub bytes: i64,
    pub cycles: Vec<i64>,
    pub operands: Vec<Operand>,
    pub immediate: bool,
    pub flags: Flags,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operand {
    pub name: String,
    pub immediate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    #[serde(rename = "Z")]
    pub z: String,
    #[serde(rename = "N")]
    pub n: String,
    #[serde(rename = "H")]
    pub h: String,
    #[serde(rename = "C")]
    pub c: String,
}

// Metainformation from gameboy rom
pub struct CartridgeMetadata {
  //(None, "="), # "Native" endian.
  //entrypoint : u16, // 0x100-0x103 (entrypoint)
  //nintendo_logo : u64 , // 0x104-0x133 (nintendo logo)
  title: u128, // 0x134-0x142 (cartridge title) (0x143 is shared with the cgb flag)
  //cgb : u8, // ("cgb", 'B'), // 0x143 (cgb flag)
  new_licensee_code: u16, //("new_licensee_code", 'H'), // 0x144-0x145 (new licensee code)
  sgb : u8, //("sgb", 'B'), // 0x146 (sgb `flag)
  cartridge_type: u8,   //("cartridge_type", 'B'), // 0x147 (cartridge type)
  rom_size : u8, //("rom_size", 'B'), // 0x148 (ROM size)
  ram_size : u8, //("ram_size", 'B'), // 0x149 (RAM size)
  destination_code : u8, // 0x14A (destination code)
  old_licensee_code : u8, // 0x14B (old licensee code)
  mask_rom_version : u8, // 0x14C (mask rom version)
  header_checksum : u8, // 0x14D (header checksum)
  global_checksum: u16, // 0x14E-0x14F (global checksum)
}

impl CartridgeMetadata {
    fn from_reader(mut rdr: impl Read) -> io::Result<Self> {
        //(None, "="), # "Native" endian.
        //let entrypoint = rdr.read_u16::<BigEndian>()?; // 0x102-0x103 (entrypoint)
        //let nintendo_logo = rdr.read_u48::<BigEndian>()?;// 0x104-0x133 (nintendo logo)
        let title =  rdr.read_u128::<BigEndian>()?; // 0x134-0x142 (cartridge title) (0x143 is shared with the cgb flag)
        //let cgb =  rdr.read_u8()?;// ("cgb", 'B'), // 0x143 (cgb flag)
        let new_licensee_code = rdr.read_u16::<BigEndian>()?; //("new_licensee_code", 'H'), // 0x144-0x145 (new licensee code)
        let sgb = rdr.read_u8()?; //("sgb", 'B'), // 0x146 (sgb `flag)
        let cartridge_type = rdr.read_u8()?;   //("cartridge_type", 'B'), // 0x147 (cartridge type)
        let rom_size = rdr.read_u8()?; //("rom_size", 'B'), // 0x148 (ROM size)
        let ram_size = rdr.read_u8()?; //("ram_size", 'B'), // 0x149 (RAM size)
        let destination_code = rdr.read_u8()?; // 0x14A (destination code)
        let old_licensee_code = rdr.read_u8()?; // 0x14B (old licensee code)
        let mask_rom_version = rdr.read_u8()?; // 0x14C (mask rom version)
        let header_checksum = rdr.read_u8()?; // 0x14D (header checksum)
        let global_checksum = rdr.read_u16::<BigEndian>()?; // 0x14E-0x14F (global checksum)
        

        Ok(CartridgeMetadata {
            //entrypoint,
            //nintendo_logo,
            title,
            //cgb,
            new_licensee_code,
            sgb,
            cartridge_type,
            rom_size,
            ram_size,
            destination_code,
            old_licensee_code,
            mask_rom_version,
            header_checksum,
            global_checksum,
        })
    }
}



// get ram size
pub fn ram_size (value: u8) -> String {
    /*0x00: 0x00 * 0x400,       #  None
    0x01: 0x02 * 0x400,       #   2kb
    0x02: 0x08 * 0x400,       #   8kb
    0x03: 0x20 * 0x400,       #  32kb ( 4 banks)
    0x04: 0x80 * 0x400,       # 128kb (16 banks)
    0x05: 0x40 * 0x400,       #  64kb ( 8 banks)
    */
   
    match value {
        0x00 => return "None".to_string(),
        0x01 => return "2kb".to_string(),
        0x02 => return "8kb".to_string(),
        0x03 => return "32kb".to_string(),
        0x04 => return "128kb".to_string(),
        0x05 => return "64kb".to_string(),
        _ => return "ERROR".to_string(),
    }
    
}

pub fn rom_size (value: u8) -> i32 {
    const BANK_SIZE : i32 = 0x4000;

    /*
        0x00: BANK_SIZE * 0x002,  # no ROM banking
        0x01: BANK_SIZE * 0x004,
        0x02: BANK_SIZE * 0x008,
        0x03: BANK_SIZE * 0x010,
        0x04: BANK_SIZE * 0x020,
        0x05: BANK_SIZE * 0x040,  # only 0x3f banks used by MBC1
        0x06: BANK_SIZE * 0x080,  # only 0x7d banks used by MBC1
        0x07: BANK_SIZE * 0x100,
        0x08: BANK_SIZE * 0x200,
        0x52: BANK_SIZE * 0x048,
        0x53: BANK_SIZE * 0x050,
        0x54: BANK_SIZE * 0x060,
    */
    match value {
        0x00 => return 0x002 * BANK_SIZE / 1024,
        0x01 => return 0x004 * BANK_SIZE / 1024,
        0x02 => return 0x008 * BANK_SIZE / 1024,
        0x03 => return 0x010 * BANK_SIZE / 1024,
        0x04 => return 0x020 * BANK_SIZE / 1024,
        0x05 => return 0x040 * BANK_SIZE / 1024,
        0x06 => return 0x080 * BANK_SIZE / 1024,
        0x07 => return 0x100 * BANK_SIZE / 1024,
        0x08 => return 0x200 * BANK_SIZE / 1024,
        0x52 => return 0x048 * BANK_SIZE / 1024,
        0x53 => return 0x050 * BANK_SIZE / 1024,
        0x54 => return 0x060 * BANK_SIZE / 1024,
        _ => return -1,
    }

}

// Convert number to hex string.
fn to_hex (value: u8) -> String {
    let value = if value > 255 { 255 } else if value < 0 { 0 } else { value };
    format!("0x{:02X}", value)
}

fn read_cartridge_metadata(mut buffer: &File) -> CartridgeMetadata {
    /*
    Unpacks the cartridge metadata from `buffer` at `offset` and
    returns a `CartridgeMetadata` object.
    */
    //data = struct.unpack_from(CARTRIDGE_HEADER, buffer, offset=offset)
    //return CartridgeMetadata._make(data)

    //buffer.seek(SeekFrom::Start(ENTRYPOINT_OFFSET));
    //let entrypoint = buffer.take(2);
    //println!(entrypoint);
    buffer.seek(SeekFrom::Start(HEADER_START));

    let cat_metadata =  CartridgeMetadata::from_reader(buffer).unwrap();
    //let out = cm.read_exact(buffer).unwrap();
    //println!("{:?}", cat_metadata);
    //println!("Read metadata: {:#?}", cat_metadata);
    buffer.seek(SeekFrom::Start(0x00));
    return cat_metadata;
}

fn parse_rom(mut buffer: &File) {

    let cart_type_json= fs::read_to_string("./data/Cardtypes.json")
    .expect("Unable to read file");

    let cart_types_map: HashMap<String, String> =  serde_json::from_str(&cart_type_json).unwrap();

    let old_licensee_json= fs::read_to_string("./data/old_licensee.json")
    .expect("Unable to read file");

    let old_licensee_map: HashMap<String, String> =  serde_json::from_str(&old_licensee_json).unwrap();
    
    let mut cgb = false;
    // read the metadata of rom
    let metadata : CartridgeMetadata = read_cartridge_metadata(&buffer);


    let title_bytes = metadata.title.to_be_bytes();
    let str_title = str::from_utf8(&title_bytes).unwrap();

    println!("Title: {}", str_title);
    if (metadata.old_licensee_code != 0x33) {
        println!("Publisher: {}",old_licensee_map.get(&to_hex(metadata.old_licensee_code)).unwrap());
    } else {
        //println!("Publisher: {}",new_licensee_map.get(&to_hex(metadata.new_licensee_code)).unwrap());
        cgb = true;
    }
    println!("Cartridge type: {}",cart_types_map.get(&to_hex(metadata.cartridge_type)).unwrap());
    println!("Destination code (Japanese Version): {}",metadata.destination_code == 0x00);
    // if old-licensee = 0x33: 'GBC_GAME',
    // use new-licensee TODO: if statement for this cases
    println!("Super GameBoy: {}",metadata.sgb == 0x00);
    println!("Color GameBoy: {}", cgb);
    println!("RAM size: {}",ram_size( metadata.ram_size ));
    println!("ROM size: {}kB",rom_size( metadata.rom_size ));
    println!("Global checksum: {:X}",metadata.global_checksum);
    println!("Header checksum: {:X}",metadata.header_checksum);
    //println!("Nintendo Logo: {:X}",metadata.nintendo_logo);
    //println!("entrypoint: {:X}",metadata.entrypoint);
}

fn dissass_rom(mut buffer: File) {

    // parse the rom
    parse_rom(&buffer);

    let mut reader = BufReader::new(buffer);
    let mut rom = Vec::new();
    
    // Read file into vector.
    reader.read_to_end(&mut rom);

    let ep = u16::from_le_bytes(rom[0x102..0x104].try_into().unwrap());
    println!("Entrypoint: 0x{:X}", ep);
    

    let unprefixed_data = fs::read_to_string("./data/unprefixed.json")
    .expect("Unable to read file");

    let prefixed_data = fs::read_to_string("./data/prefixed.json")
    .expect("Unable to read file");

    let map: HashMap<String, Instruction> =  serde_json::from_str(&unprefixed_data).unwrap();
    let prefix_map: HashMap<String, Instruction> =  serde_json::from_str(&prefixed_data).unwrap();
    
    // start dissasembling
    let mut pc: usize = usize::from(ep);
    
    while pc < rom.len() {
        let mut ins : &Instruction = map.get(&to_hex(rom[pc])).unwrap();

        // check for prefix
        if (ins.mnemonic == "PREFIX") {
            ins = prefix_map.get(&to_hex(rom[pc])).unwrap();
        }

        let mut bytes = Vec::new();
        let mut n : usize = 0;
        let bytesize : usize = usize::try_from(ins.bytes).unwrap();
    
        //println!("Bytesize: {:?}",bytesize);
        while n < bytesize {
            bytes.insert(n, rom[pc+n]); 
            //println!("{:X}", rom[pc+n]);
            n += 1;
        
        }

        print!("{:x}: \t",pc );
        //print!("len({:X}) : ", ins.bytes);

        for byte in &bytes {
            print!(" {:x}", byte);
        }
       
        if bytesize == 3 {
            print!(" \t{}", ins.mnemonic);
        } else {
            print!(" \t\t{}", ins.mnemonic);
        }
        

        // parse operands
        let ops = &ins.operands;

        let cnt = 1;
        for operand in ops {
    
            let opslice = &bytes[1..];
            let opsize = usize::try_from(operand.bytes.unwrap_or(0x0)).unwrap();
            let mut i:usize = opsize;
            if opsize > 0 {
                print!(" 0x");
                while i != 0  {
                    print!("{:x}", &opslice[i-1]);
                    i -= 1;
                }
            } else {
                print!(" {}", operand.name);
            }

        }
        println!("");
        pc += bytesize;
    }   

}

fn main() {
   
    let file = File::open("testdata/rom.gb").unwrap();
    //parse_rom(file);

    dissass_rom(file);
    
}