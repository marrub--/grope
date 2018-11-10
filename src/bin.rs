//! Binary data conversion utilities.

pub type Ident = [u8; 4];

pub type ResultS<T> = Result<T, &'static str>;

pub trait BinToNum
{
   // Checked
   fn c_iden(&self, i: usize) -> ResultS<Ident>;
   fn c_u32b(&self, i: usize) -> ResultS<u32>;
   fn c_u16b(&self, i: usize) -> ResultS<u16>;

   fn c_i32b(&self, i: usize) -> ResultS<i32>
      {match self.c_u32b(i) {Ok(n) => Ok(n as i32), Err(e) => Err(e)}}
   fn c_i16b(&self, i: usize) -> ResultS<i16>
      {match self.c_u16b(i) {Ok(n) => Ok(n as i16), Err(e) => Err(e)}}

   // Optional
   fn o_iden(&self, i: usize) -> Option<Ident> {self.c_iden(i).ok()}
   fn o_u32b(&self, i: usize) -> Option<u32>   {self.c_u32b(i).ok()}
   fn o_u16b(&self, i: usize) -> Option<u16>   {self.c_u16b(i).ok()}
   fn o_i32b(&self, i: usize) -> Option<i32>   {self.c_i32b(i).ok()}
   fn o_i16b(&self, i: usize) -> Option<i16>   {self.c_i16b(i).ok()}

   // Unchecked
   fn b_iden(&self, i: usize) -> Ident {self.c_iden(i).unwrap()}
   fn b_u32b(&self, i: usize) -> u32   {self.c_u32b(i).unwrap()}
   fn b_u16b(&self, i: usize) -> u16   {self.c_u16b(i).unwrap()}
   fn b_i32b(&self, i: usize) -> i32   {self.c_i32b(i).unwrap()}
   fn b_i16b(&self, i: usize) -> i16   {self.c_i16b(i).unwrap()}
}

impl BinToNum for [u8]
{
   fn c_iden(&self, i: usize) -> ResultS<Ident>
   {
      if i + 3 >= self.len() {return Err("not enough data")}
      Ok([self[i], self[i+1], self[i+2], self[i+3]])
   }

   fn c_u32b(&self, i: usize) -> ResultS<u32>
   {
      if i + 3 >= self.len() {return Err("not enough data")}
      Ok(self[i  ] as (u32) << 24 | self[i+1] as (u32) << 16 |
         self[i+2] as (u32) <<  8 | self[i+3] as (u32))
   }

   fn c_u16b(&self, i: usize) -> ResultS<u16>
   {
      if i + 1 >= self.len() {return Err("not enough data")}
      Ok(self[i] as (u16) << 8 | self[i+1] as (u16))
   }
}

pub fn d_u32b(n: u32) -> [u8; 4] {[(n >> 24) as u8, (n >> 16) as u8,
                                   (n >>  8) as u8, (n >>  0) as u8]}
pub fn d_u16b(n: u16) -> [u8; 2] {[(n >>  8) as u8, (n >>  0) as u8]}
pub fn d_i32b(n: i32) -> [u8; 4] {d_u32b(n as u32)}
pub fn d_i16b(n: i16) -> [u8; 2] {d_u16b(n as u16)}

// EOF
