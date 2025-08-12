pub struct SFC32 {
    pub state: u128,
}

impl SFC32 {
    pub fn next_value(&mut self) -> u32 {
        let [mut a, mut b, mut c, mut d] = from_u128_to_u32s(self.state);
        let t: u32 = a.wrapping_add(b).wrapping_add(d);
        d = d.wrapping_add(1);
        a = b ^ b.wrapping_shr(9);
        b = c ^ c.wrapping_shl(3);
        c = c.wrapping_shl(21) ^ c.wrapping_shr(11);
        c = c.wrapping_add(t);
        self.state = from_u32s_to_u128([a, b, c, d]);
        t
    }
}

fn from_u128_to_u32s(from: u128) -> [u32; 4] {
    let out = from.to_le_bytes();
    let a = u32::from_le_bytes([out[0], out[1], out[2], out[3]]);
    let b = u32::from_le_bytes([out[4], out[5], out[6], out[7]]);
    let c = u32::from_le_bytes([out[8], out[9], out[10], out[11]]);
    let d = u32::from_le_bytes([out[12], out[13], out[14], out[15]]);
    [a, b, c, d]
}

fn from_u32s_to_u128(from: [u32; 4]) -> u128 {
    let a = u32::to_le_bytes(from[0]);
    let b = u32::to_le_bytes(from[1]);
    let c = u32::to_le_bytes(from[2]);
    let d = u32::to_le_bytes(from[3]);
    u128::from_le_bytes([
        a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3], d[0], d[1], d[2],
        d[3],
    ])
}
