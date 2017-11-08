use ffi;
use into_ruby::*;
use super::Attribute;
use util::*;

impl IntoRuby for Attribute {
    unsafe fn class() -> ffi::VALUE {
        ATTRIBUTE.unwrap()
    }

    unsafe fn mark(&self) {}
}

static mut ATTRIBUTE: Option<ffi::VALUE> = None;

pub unsafe fn init() {
    let attribute = ffi::rb_define_class_under(::module(), cstr!("Attribute"), ffi::rb_cObject);
    ATTRIBUTE = Some(attribute);

    ffi::rb_define_singleton_method(
        attribute,
        cstr!("from_database"),
        from_database as *const _,
        3,
    );

    ffi::rb_define_method(attribute, cstr!("value"), value as *const _, 0);
}

extern "C" fn from_database(
    _class: ffi::VALUE,
    name: ffi::VALUE,
    value: ffi::VALUE,
    ty: ffi::VALUE,
) -> ffi::VALUE {
    Attribute::from_database(name, value, ty).into_ruby()
}

extern "C" fn value(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.value()
}
