use ffi;
use into_ruby::*;
use super::Attribute;

impl IntoRuby for Attribute {
    unsafe fn class() -> ffi::VALUE {
        ATTRIBUTE.unwrap()
    }

    unsafe fn mark(&self) {
    }
}

static mut ATTRIBUTE: Option<ffi::VALUE> = None;

pub unsafe fn init() {
    let attribute = ffi::rb_define_class_under(::module(), cstr!("Attribute"), ffi::rb_cObject);
    ATTRIBUTE = Some(attribute);

    ffi::rb_define_alloc_func(attribute, Some(Attribute::allocate));
}
