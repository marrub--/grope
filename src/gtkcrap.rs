use crate::err;
use glib::object::Downcast;
use glib::translate::{FromGlibPtrNone, Stash, ToGlib, ToGlibPtr};
use gtk::{FileChooserAction, prelude::*};
use gtk_sys::*;
use libc::{c_char, c_void};
use std::ptr;

pub fn file_chooser(title: &str, action: FileChooserAction, parent: &gtk::Window) -> gtk::FileChooserDialog
{
   unsafe
   {
      let title = Some(title);
      let title = title.to_glib_none();
      let parent = parent.to_glib_none();
      let first = Some("_Cancel");
      let first = first.to_glib_none();
      let second = Some(match action {
      FileChooserAction::Open         => "_Open",
      FileChooserAction::Save         => "_Save",
      FileChooserAction::SelectFolder => "_Select",
      FileChooserAction::CreateFolder => "_Create",
      FileChooserAction::__Unknown(_) => "_Choose",
      });
      let second: Stash<*const c_char, Option<&str>> = second.to_glib_none();
      gtk::Widget::from_glib_none(gtk_file_chooser_dialog_new(
         title.0, parent.0,
         action.to_glib(),
         first.0, GTK_RESPONSE_CANCEL,
         second.0, GTK_RESPONSE_ACCEPT,
         ptr::null() as *const c_void)).downcast_unchecked()
   }
}

pub fn err_dlg(w: &gtk::Window, r: Result<(), err::StrErr>)
{
   match r {
   Ok(()) => (),
   Err(e) => {
      let dlg = gtk::MessageDialog::new(Some(w),
                                        gtk::DialogFlags::MODAL,
                                        gtk::MessageType::Error,
                                        gtk::ButtonsType::Ok,
                                        &e.0);
      dlg.run();
      dlg.destroy();
   }
   }
}

// EOF
