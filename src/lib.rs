use std::io::{Error, ErrorKind};
use std::io::Read;
pub struct VarInt {}
impl VarInt {
    pub fn write_to_bytes(value: i32) -> Vec<u8> {
        let mut value = value as u32;
        if value == 0 {
            return vec![0];
        }
        if value <= 127 && value > 0 {
            return vec![value as u8];
        }
        if value <= 255 && value > 0 {
            return vec![value as u8, 1];
        }
        let mut out = vec![];
        while value != 0 {
            let currentbyte = value & 0b01111111;
            let mut currentbyte = currentbyte as u8;
            value >>= 7;
            if value != 0 {
                currentbyte |= 0b10000000;
            }
            out.push(currentbyte);
        }
        return out;
    }
    pub fn read_from_bytes(mut bytes: Vec<u8>) -> std::io::Result<i32> {
        bytes.reverse();
        let mut value: i32 = 0;
        let mut bitoffset = 0;
        let mut currentbyte = 0;
        let mut set = true;
        while (currentbyte & 0b10000000) != 0 || set == true {
            if bitoffset == 35 {
                return Err(Error::new(ErrorKind::Other, "VarInt too large!"));
            }
            currentbyte = bytes
                .pop()
                .ok_or_else(|| Error::new(ErrorKind::Other, "Not enough bits left!"))?
                as i32;
            value |= (currentbyte & 0b01111111) << bitoffset;
            bitoffset += 7;
            set = false;
        }
        return Ok(value);
    }
    pub fn read_from_reader(reader: &mut dyn std::io::Read) -> std::io::Result<i32> {
        let mut value: i32 = 0;
        let mut bitoffset = 0;
        let mut currentbyte = 0;
        let mut set = true;
        while (currentbyte & 0b10000000) != 0 || set == true {
            if bitoffset == 35 {
                return Err(Error::new(ErrorKind::Other, "VarInt too large!"));
            }
            currentbyte = Self::read_byte(reader)? as i32;
            value |= (currentbyte & 0b01111111) << bitoffset;
            bitoffset += 7;
            set = false;
        }
        return Ok(value);
    }
    fn read_byte(stream: &mut dyn std::io::Read) -> std::io::Result<u8> {
        let mut x = [0; 1];
        stream.read_exact(&mut x)?;
        return Ok(x[0]);
    }
}
pub struct PacketUtils {}
impl PacketUtils {
    pub fn write_packet(packetid: usize, mut packet: Vec<u8>) -> Vec<u8> {
        let mut vec = vec![];
        let mut id = VarInt::write_to_bytes(packetid as i32);
        vec.append(&mut VarInt::write_to_bytes(
            (id.len() + packet.len()) as i32,
        ));
        vec.append(&mut id);
        vec.append(&mut packet);
        return vec;
    }
    pub fn write_packet_lengthless(packetid: usize, mut packet: Vec<u8>) -> Vec<u8> {
        let mut vec = vec![];
        let mut id = VarInt::write_to_bytes(packetid as i32);
        vec.append(&mut id);
        vec.append(&mut packet);
        return vec;
    }
    pub fn write_string(string: String) -> Vec<u8> {
        let mut vec = string.as_bytes().to_vec();
        vec.reverse();
        let mut x = VarInt::write_to_bytes(vec.len() as i32);
        x.reverse();
        vec.append(&mut x);
        vec.reverse();
        return vec;
    }
    pub fn write_compressed_packet(packetid: usize, packet: Vec<u8>, threshold: i32) -> std::io::Result<Vec<u8>> {
        use deflate::deflate_bytes_zlib;
        let mut data = Self::write_packet_lengthless(packetid, packet);
        let mut compress = false;
        if threshold < 0 {
            return Err(Error::new(ErrorKind::Other, "Compression not enabled!"));
        }
        if data.len() >= threshold as usize {
            compress = true;
        }
        if compress == false {
            let mut packet = vec![];
            packet.append(&mut VarInt::write_to_bytes(data.len() as i32 + 1));
            packet.append(&mut VarInt::write_to_bytes(0x00));
            packet.append(&mut data);
            return Ok(packet);
        } else {
            let mut packet = vec![];
            let datalen = data.len().clone();
            let mut data = deflate_bytes_zlib(&data);
            let mut data = compressed;
            packet.append(&mut VarInt::write_to_bytes(datalen as i32));
            packet.append(&mut data);
            packet.reverse();
            let mut x = VarInt::write_to_bytes(packet.len() as i32);
            x.reverse();
            packet.append(&mut x);
            packet.reverse();
            return Ok(packet);
        }
    }
    pub fn read_compressed_packet(reader: &mut dyn std::io::Read) -> std::io::Result<(usize, Vec<u8>)> {
        let packet = Self::read_varint_prefixed_bytearray(reader)?;
        let mut reader = std::io::Cursor::new(packet);
        let dtlvint = VarInt::read_from_reader(&mut reader)?;
        match dtlvint {
            0x00 => {
                let packetid = VarInt::read_from_reader(&mut reader)?;
                let mut packet = vec![];
                reader.read_to_end(&mut packet)?;
                return Ok((packetid as usize, packet));
            }
            len => {
                use compress::zlib;
                let mut decompressed = Vec::new();
                let mut bytes = vec![];
                reader.read_to_end(&mut bytes)?;
                let mut bytes = std::io::Cursor::new(bytes); 
                zlib::Decoder::new(bytes).read_to_end(&mut decompressed)?;
                if decompressed.len() != len as usize {
                    return Err(Error::new(ErrorKind::Other, "Decompression has failed!"));
                }
                let mut reader = std::io::Cursor::new(decompressed);
                let packetid = VarInt::read_from_reader(&mut reader)?;
                let mut packet = vec![];
                reader.read_to_end(&mut packet)?;
                return Ok((packetid as usize, packet));
            }
        }
    }
    pub fn read_varint_prefixed_bytearray(
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Vec<u8>> {
        let mut vec = vec![0; VarInt::read_from_reader(reader)? as usize];
        reader.read_exact(&mut vec)?;
        Ok(vec)
    }
}
pub enum Element {
    StringElement { string: String },
    VarintBytearray { array: Vec<u8> },
    UnsignedByte { byte: u8 },
    Byte { byte: i8 },
    VarInt { varint: i32 },
    Short { short: i16 },
    UnsignedShort { short: u16 },
    Int { int: i32 },
    Long { long: i64 },
    Float { float: f32 },
    Double { double: f64 },
}
pub struct PacketConstructor {
    elements: Vec<Element>,
}
impl PacketConstructor {
    pub fn new() -> Self {
        return Self {
            elements: Vec::new(),
        };
    }
    pub fn insert_string(&mut self, string: &str) {
        self.elements.push(Element::StringElement {
            string: string.to_string(),
        });
    }
    pub fn insert_bytearray(&mut self, array: Vec<u8>) {
        self.elements
            .push(Element::VarintBytearray { array: array });
    }
    pub fn insert_unsigned_byte(&mut self, byte: u8) {
        self.elements.push(Element::UnsignedByte { byte: byte });
    }
    pub fn insert_byte(&mut self, byte: i8) {
        self.elements.push(Element::Byte { byte: byte });
    }
    pub fn insert_short(&mut self, short: i16) {
        self.elements.push(Element::Short { short: short });
    }
    pub fn insert_unsigned_short(&mut self, short: u16) {
        self.elements.push(Element::UnsignedShort { short: short });
    }
    pub fn insert_int(&mut self, int: i32) {
        self.elements.push(Element::Int { int: int });
    }
    pub fn insert_long(&mut self, long: i64) {
        self.elements.push(Element::Long { long: long });
    }
    pub fn insert_float(&mut self, float: f32) {
        self.elements.push(Element::Float { float: float });
    }
    pub fn insert_double(&mut self, double: f64) {
        self.elements.push(Element::Double { double: double });
    }
    pub fn insert_varint(&mut self, varint: i32) {
        self.elements.push(Element::VarInt { varint: varint });
    }
    pub fn insert_bool(&mut self, value: bool) {
        let byte = match value {
            true => 1,
            false => 0,
        };
        self.elements.push(Element::UnsignedByte { byte: byte });
    }
    pub fn build(self, id: usize) -> Vec<u8> {
        let mut packet = vec![];
        for element in self.elements {
            match element {
                Element::StringElement { string } => {
                    packet.append(&mut PacketUtils::write_string(string));
                }
                Element::VarintBytearray { mut array } => {
                    packet.append(&mut VarInt::write_to_bytes(array.len() as i32));
                    packet.append(&mut array);
                }
                Element::UnsignedByte { byte } => {
                    packet.push(byte);
                }
                Element::Byte { byte } => {
                    packet.push(byte.to_le_bytes()[0]);
                }
                Element::VarInt { varint } => {
                    packet.append(&mut VarInt::write_to_bytes(varint));
                }
                Element::Short { short } => {
                    packet.append(&mut short.to_be_bytes().to_vec());
                }
                Element::UnsignedShort { short } => {
                    packet.append(&mut short.to_be_bytes().to_vec());
                }
                Element::Int { int } => {
                    packet.append(&mut int.to_be_bytes().to_vec());
                }
                Element::Long { long } => {
                    packet.append(&mut long.to_be_bytes().to_vec());
                }
                Element::Float { float } => {
                    packet.append(&mut float.to_be_bytes().to_vec());
                }
                Element::Double { double } => {
                    packet.append(&mut double.to_be_bytes().to_vec());
                }
            }
        }
        let packet = PacketUtils::write_packet(id, packet);
        return packet;
    }
}
