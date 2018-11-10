fn crc_init() -> [u32; 256]
{
   let mut t = [0; 256];
   for n in 0..256
   {
      t[n] = (0..8).fold(n as u32, |a, _|
         {if a & 1 == 1 {0xedb88320 ^ a >> 1} else {a >> 1}});
   }
   t
}

pub fn crc32(b: &[u8], s: u32) -> u32
{
   let t = crc_init();
   !b.iter().fold(!s, |a, &o| {a >> 8 ^ t[(a & 0xff ^ o as u32) as usize]})
}

// EOF
