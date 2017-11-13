use ffi;
use into_ruby::{Allocate, IntoRuby};
use super::AttributeSet;
use util::*;

impl IntoRuby for AttributeSet {
    unsafe fn class() -> ffi::VALUE {
        ATTRIBUTE_SET.unwrap()
    }

    unsafe fn mark(&self) {
        for (key, value) in &self.attributes {
            let sym = ffi::rb_id2sym(*key);
            ffi::rb_gc_mark(sym);
            value.mark()
        }
    }
}

static mut ATTRIBUTE_SET: Option<ffi::VALUE> = None;

pub unsafe fn init() {
    let attribute_set =
        ffi::rb_define_class_under(::module(), cstr!("AttributeSet"), ffi::rb_cObject);
    ATTRIBUTE_SET = Some(attribute_set);

    ffi::rb_define_alloc_func(attribute_set, Some(AttributeSet::allocate));

    ffi::rb_define_method(attribute_set, cstr!("[]"), get as *const _, 1);
    ffi::rb_define_method(attribute_set, cstr!("write_from_database"), write_from_database as *const _, 2);
    ffi::rb_define_method(attribute_set, cstr!("initialize_copy"), initialize_copy as *const _, 1);
}

extern "C" fn get(this: ffi::VALUE, key: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = unsafe { ffi::rb_sym2id(key) };
    this.get(key)
        .map(IntoRuby::as_ruby)
        .unwrap_or(unsafe { ffi::Qnil })
}

extern "C" fn write_from_database(this: ffi::VALUE, key: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = unsafe { ffi::rb_sym2id(key) };
    this.write_from_database(key, value);
    unsafe { ffi::Qnil }
}

extern "C" fn initialize_copy(this_ptr: ffi::VALUE, other: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this_ptr) };
    let other = unsafe { get_struct::<AttributeSet>(other) };
    this.clone_from(other);
    this_ptr
}
