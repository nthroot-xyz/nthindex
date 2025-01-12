pub type Address = [u8; 20];

pub fn address_from_string(addr: &String) -> Address {
    let mut address_bytes: [u8; 20] = [0; 20];
    hex::decode_to_slice(addr, &mut address_bytes).expect("Decoding failed");
    address_bytes
}
