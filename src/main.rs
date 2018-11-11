#![windows_subsystem="windows"]

#[allow(dead_code)] mod bin;
#[allow(dead_code)] mod crc;
                    mod err;
                    mod gtkcrap;
                    mod icon;
#[allow(dead_code)] mod png;

use cairo::ImageSurface;
use crate::{bin::*, err::*, gtkcrap::*, png::*};
use gdk::{EventMask, enums::key};
use gtk::{FileChooserAction, prelude::*};
use memmap::Mmap;
use std::{cell::RefCell, fs::File, path::{Path, PathBuf}, rc::Rc};

fn drag(e: &gdk::EventMotion, v: &mut Stuff) -> Inhibit
{
   if e.get_state().contains(gdk::ModifierType::BUTTON3_MASK)
   {
      let p = e.get_position();

      if let Some(c) = &mut v.c
      {
         v.v.0 -= (c.0 - p.0) / 2.0;
         v.v.1 -= (c.1 - p.1) / 2.0;
         v.update();
      }

      v.c = Some(p);

      Inhibit(true)
   }
   else if e.get_state().contains(gdk::ModifierType::BUTTON1_MASK)
   {
      if let None = v.png {return Inhibit(false)}

      let p = e.get_position();

      if let Some(c) = &mut v.c
      {
         v.p.0 += (c.0 - p.0) as i32;
         v.p.1 += (c.1 - p.1) as i32;
         v.update();
      }

      v.c = Some(p);

      Inhibit(true)
   }
   else
      {Inhibit(false)}
}

fn drop(_e: &gdk::EventButton, v: &mut Stuff) -> Inhibit
{
   v.c = None;

   Inhibit(false)
}

fn draw(c: &cairo::Context, v: &mut Stuff) -> Inhibit
{
   const VW: f64 = 320.0;
   const VH: f64 = 200.0;
   const HW: f64 = 160.0;
   const HH: f64 = 100.0;

   // scale to 320x200 (1.2x height)
   c.scale(1.0, 1.2);

   // draw background
   c.set_source_rgb(167.0/255.0, 107.0/255.0, 107.0/255.0);
   c.rectangle(0.0, 0.0, VW, VH);
   c.fill();

   // center
   c.translate(HW + v.v.0, HH + v.v.1);

   // guide lines
   c.set_source_rgb(0.5, 0.0, 0.0);
   c.set_line_width(1.0);

   // center
   c.move_to(-VW, 0.0); c.line_to( VW, 0.0); c.stroke();
   c.move_to(0.0, -VH); c.line_to(0.0,  VH); c.stroke();

   // hud borders
   c.move_to(0.0,  VH); c.line_to(VW, VH); c.stroke();
   c.move_to( VW, 0.0); c.line_to(VW, VH); c.stroke();

   // statusbar
   c.move_to(0.0, VH-32.0); c.line_to(VW, VH-32.0); c.stroke(); // doom
   c.move_to(0.0, VH-38.0); c.line_to(VW, VH-38.0); c.stroke(); // hexen
   c.move_to(0.0, VH-42.0); c.line_to(VW, VH-42.0); c.stroke(); // heretic

   // draw image
   if let Some(img) = &v.img
   {
      c.set_source_surface(img, 0.0 - v.p.0 as f64, 0.0 - v.p.1 as f64);
      c.paint();
   }

   Inhibit(false)
}

fn key(k: &gdk::EventKey, v: &mut Stuff) -> Inhibit
{
   if let None = v.png {return Inhibit(false)}
   match k.get_keyval() {
   key::Left  => {v.p.0 += 1; v.update(); Inhibit(true)},
   key::Right => {v.p.0 -= 1; v.update(); Inhibit(true)},
   key::Up    => {v.p.1 += 1; v.update(); Inhibit(true)},
   key::Down  => {v.p.1 -= 1; v.update(); Inhibit(true)},
   _          => Inhibit(false),
   }
}

fn open(v: &mut Stuff)
{
   let fc = file_chooser("Open Image", FileChooserAction::Open, &v.win);

   if fc.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
      if let Some(p) = fc.get_filename() {
         let r = v.set_img(&p); err_dlg(&v.win, r);
      }
   }

   fc.destroy();
}

fn save(v: &mut Stuff)
   {if let Some(p) = &v.path {let r = v.save_img(&p); err_dlg(&v.win, r)}}

fn save_as(v: &mut Stuff)
{
   let fc = file_chooser("Save Image", FileChooserAction::Save, &v.win);

   if fc.run() == gtk_sys::GTK_RESPONSE_ACCEPT {
      if let Some(p) = fc.get_filename() {
         let r = v.save_img(&p); err_dlg(&v.win, r);
      }
   }

   fc.destroy();
}

#[allow(dead_code)]
struct Stuff
{
   p: (i32, i32),
   s: (u32, u32),
   png: Option<PngFile>,
   img: Option<ImageSurface>,

   v:        (f64, f64),
   c: Option<(f64, f64)>,

   path: Option<PathBuf>,

   lx:   gtk::Label,
   ly:   gtk::Label,
   xh:   gtk::Box,
   area: gtk::DrawingArea,
   xv:   gtk::Box,
   win:  gtk::Window,
}

impl Stuff
{
   fn update(&self)
   {
      self.lx.set_text(&(-self.p.0).to_string());
      self.ly.set_text(&(-self.p.1).to_string());
      self.area.queue_draw();
   }

   fn set_img(&mut self, fname: &Path) -> Result<(), StrErr>
   {
      let mut fp  = File::open(fname)?;
      let     png = PngFile::new(&unsafe{Mmap::map(&fp)?})?;
      let     img = ImageSurface::create_from_png(&mut fp)?;

      let xy =
         if let Some(xy) = png.chunk(*b"grAb", |c| (c.b_i32b(0), c.b_i32b(4)))
            {xy}
         else
            {(0, 0)};

      let wh = png.chunk(*b"IHDR", |c| (c.b_u32b(0), c.b_u32b(4)))
         .ok_or(StrErr::new("no IHDR"))?;

      self.p    = xy;
      self.s    = wh;
      self.png  = Some(png);
      self.img  = Some(img);
      self.path = Some(fname.canonicalize()?);

      self.update();

      Ok(())
   }

   fn save_img(&self, fname: &Path) -> Result<(), StrErr>
   {
      match &self.png {
      Some(png) => {
         let mut png = png.clone();
         let mut   c = PngChunk{typ: *b"grAb", dat: Vec::new()};
         for b in &d_i32b(self.p.0) {c.dat.push(*b)}
         for b in &d_i32b(self.p.1) {c.dat.push(*b)}
         if let Some(n) = png.find_chunk(*b"grAb") {png.chnk[n] = c}
         else                                      {png.chnk.insert(1, c)}
         png.write(&mut File::create(fname)?)?;
         Ok(())
      }
      None => Err("no image loaded".into())
      }
   }
}

fn btn<F>(b: &gtk::Button, r: Rc<RefCell<Stuff>>, f: F)
   where F: Fn(&mut Stuff) + 'static
{
   b.connect_clicked(move |_|
      {if let Ok(mut v) = r.try_borrow_mut() {f(&mut v)}});
}

fn cb<T, U: gtk::WidgetExt>(r: Rc<RefCell<Stuff>>,
   f: impl Fn(&T, &mut Stuff) -> Inhibit + 'static)
      -> impl Fn(&U, &T) -> Inhibit + 'static
{
   move |_, e| {
      if let Ok(mut v) = r.try_borrow_mut() {f(e, &mut v)}
      else                                  {Inhibit(false)}
   }
}

fn main() -> Result<(), StrErr>
{
   gtk::init()?;

   let ly = gtk::Label::new(None);
   let lx = gtk::Label::new(None);

   let bs = gtk::Button::new_with_mnemonic("_As");
   let bq = gtk::Button::new_with_mnemonic("_Save");
   let bo = gtk::Button::new_with_mnemonic("_Open");

   let xh = gtk::Box::new(gtk::Orientation::Horizontal, 4);
   xh.add(&bo);
   xh.add(&bq);
   xh.add(&bs);
   xh.add(&lx);
   xh.add(&ly);

   let area = gtk::DrawingArea::new();
   area.set_size_request(320, 240);

   let xv = gtk::Box::new(gtk::Orientation::Vertical, 4);
   xv.add(&area);
   xv.add(&xh);

   let win = gtk::Window::new(gtk::WindowType::Toplevel);
   win.set_title("grAb editor");
   win.set_resizable(false);
   win.set_icon(Some(&gdk_pixbuf::Pixbuf::new_from_xpm_data(&icon::ICON)));
   win.add(&xv);

   let v = Rc::new(RefCell::new(
      Stuff{p:(0,0),s:(0,0),v:(0.0,0.0),c:None,
            path:None, png:None,img:None, lx,ly, xh,xv, area,win}));

   btn(&bo, v.clone(), open);
   btn(&bq, v.clone(), save);
   btn(&bs, v.clone(), save_as);

   {
      let vr = &mut *v.borrow_mut();
      vr.update();

      vr.area.connect_draw               (cb(v.clone(), draw));
      vr.win.connect_key_press_event     (cb(v.clone(), key));
      vr.win.connect_motion_notify_event (cb(v.clone(), drag));
      vr.win.connect_button_release_event(cb(v.clone(), drop));
      vr.win.connect_delete_event(|_, _| {gtk::main_quit(); Inhibit(false)});
      vr.win.set_events((EventMask::KEY_PRESS_MASK |
                         EventMask::BUTTON1_MOTION_MASK |
                         EventMask::BUTTON3_MOTION_MASK |
                         EventMask::BUTTON_RELEASE_MASK).bits() as i32);
      vr.win.show_all();
   }

   gtk::main();

   Ok(())
}

// EOF
