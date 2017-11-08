#![deny(warnings)]
extern crate libcruby_sys as ffi;
extern crate libc;

macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0").as_bytes().as_ptr() as *const ::libc::c_char
    };
}

pub mod attribute;
pub mod into_ruby;

pub fn module() -> ffi::VALUE {
    unsafe { ::MODULE }.unwrap()
}

static mut MODULE: Option<ffi::VALUE> = None;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Init_native() {
    ::MODULE = Some(ffi::rb_define_module(cstr!("RailsFastAttributes")));

    attribute::init();
}
