use ffi;
use into_ruby::IntoRuby;
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

    ffi::rb_define_method(attribute_set, cstr!("[]"), get as *const _, 1);
}

extern "C" fn get(this: ffi::VALUE, key: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = unsafe { ffi::rb_sym2id(key) };
    this.get(key)
        .map(IntoRuby::as_ruby)
        .unwrap_or(unsafe { ffi::Qnil })
}
