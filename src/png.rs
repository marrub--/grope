use crate::{bin::*, crc::*};
use std::{fs::File, io::prelude::*};

#[derive(Clone)]
#[derive(Debug)]
pub struct PngChunk
{
   pub typ: Ident,
   pub dat: Vec<u8>,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct PngFile
{
   pub chnk: Vec<PngChunk>,
}

impl PngFile
{
   pub fn new(b: &[u8]) -> Result<PngFile, &'static str>
   {
      if &b[0..8] != b"\x89PNG\r\n\x1a\n" {return Err("invalid header")}

      let mut chnk = Vec::new();
      let mut p = 8;

      loop
      {
         let len = b.c_u32b(p      )? as usize;
         let typ = b.c_iden(p+4    )?;
         let crc = b.c_u32b(p+8+len)?;

         if crc32(&b[p+4..p+8+len], 0) != crc
            {return Err("invalid CRC in chunk")}

         chnk.push(PngChunk{typ, dat: b[p+8..p+8+len].to_vec()});

         if typ == *b"IEND" {break} else {p += len + 12}
      }

      Ok(PngFile{chnk})
   }

   pub fn write(&self, fp: &mut File) -> std::io::Result<()>
   {
      fp.write_all(b"\x89PNG\r\n\x1a\n")?;
      for c in &self.chnk
      {
         fp.write_all(&d_u32b(c.dat.len() as u32))?;
         fp.write_all(&c.typ)?;
         fp.write_all(&c.dat)?;
         fp.write_all(&d_u32b(crc32(&c.dat, crc32(&c.typ, 0))))?;
      }
      Ok(())
   }

   pub fn has_chunk(&self, t: Ident) -> bool
   {
      for c in &self.chnk {if c.typ == t {return true}}
      false
   }

   pub fn find_chunk(&self, t: Ident) -> Option<usize>
   {
      for i in 0..self.chnk.len() {if self.chnk[i].typ == t {return Some(i)}}
      None
   }

   pub fn chunk<Ret, F: Fn(&[u8]) -> Ret>(&self, t: Ident, f: F) -> Option<Ret>
   {
      for c in &self.chnk {if c.typ == t {return Some(f(&c.dat))}}
      None
   }
}

// EOF
