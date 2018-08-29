#![deny(warnings)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate libcruby_sys as ffi;
extern crate indexmap;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_bytes().as_ptr() as *const ::libc::c_char
    };
}

macro_rules! id {
    ($s:expr) => {{
        lazy_static! {
            static ref MID: ::ffi::ID = unsafe { ::ffi::rb_intern(cstr!($s)) };
        }
        *MID
    }}
}

macro_rules! rstr {
    ($s:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            ::ffi::rb_utf8_str_new($s.as_ptr() as *const ::libc::c_char, $s.len() as ::libc::c_long)
        }
    }
}

pub mod attribute;
pub mod attribute_set;
pub mod builder;
pub mod into_ruby;
pub mod util;

pub fn module() -> ffi::VALUE {
    unsafe { ::MODULE }.unwrap()
}

static mut MODULE: Option<ffi::VALUE> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Init_native() {
    ::MODULE = Some(ffi::rb_define_module(cstr!("RailsFastAttributes")));

    attribute::init();
    attribute_set::init();
    builder::init();
}
