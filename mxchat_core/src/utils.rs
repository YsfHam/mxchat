use crate::io::BytesBuffer;

pub fn u16_as_bytes(value: u16) -> [u8; 2] {
    let lo = value & 0xFF;
    let hi = value >> 8;

    [hi as u8, lo as u8]
}

pub fn u32_as_bytes(value: u32) -> [u8; 4] {
    let lo_u16 = value & 0xFFFF;
    let hi_u16 = value >> 16;

    let lo_arr = u16_as_bytes(lo_u16 as u16);
    let hi_arr = u16_as_bytes(hi_u16 as u16);

    [ 
        hi_arr[0], hi_arr[1],

        lo_arr[0], lo_arr[1],
    ]
}

pub fn bytes_as_u16(bytes: &[u8; 2]) -> u16 {

    let hi = bytes[0] as u16;
    let lo = bytes[1] as u16;

    (hi << 8) | lo
}

pub fn bytes_as_u32(bytes: &[u8; 4]) -> u32 {
    let hi = bytes_as_u16(&[bytes[0], bytes[1]]) as u32;

    let lo = bytes_as_u16(&[bytes[2], bytes[3]]) as u32;

    (hi << 16) | lo
}

pub fn write_string_to_bytes_buffer(bytes_buffer: &mut BytesBuffer, data: &str) {
    let data_bytes = data.as_bytes();
    let data_len = data_bytes.len();

    bytes_buffer.write_bytes(&u32_as_bytes(data_len as u32));
    bytes_buffer.write_bytes(data_bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u16() {
        let value = 1048;

        let bytes = u16_as_bytes(value);
        println!("{bytes:?}");

        assert_eq!(value, bytes_as_u16(&bytes));
    }

    #[test]
    fn test_u32() {
        let value = 570_234;

        let bytes = u32_as_bytes(value);
        println!("{bytes:?}");

        assert_eq!(value, bytes_as_u32(&bytes));
    }
}