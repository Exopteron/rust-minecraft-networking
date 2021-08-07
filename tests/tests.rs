use rust_minecraft_networking;
#[test]
fn test1() {
    let packetc = rust_minecraft_networking::PacketUtils::write_compressed_packet(0x01, vec![87; 9521], 256).unwrap();
    let mut packetcreader = std::io::Cursor::new(packetc.clone());
    let packetd = rust_minecraft_networking::PacketUtils::read_compressed_packet(&mut packetcreader).unwrap();
    panic!("Hello, world! {:?} {:?}", packetc, packetd);
}
#[test]
fn test2() {
    let packetc = rust_minecraft_networking::VarInt::write_to_bytes(37);
    let mut packetcreader = std::io::Cursor::new(packetc.clone());
    let mut out = rust_minecraft_networking::VarInt::read_from_reader(&mut packetcreader).unwrap();
    panic!("Out: {}", out);
}