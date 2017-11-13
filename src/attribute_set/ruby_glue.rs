use attribute::Attribute;
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
    ffi::rb_define_method(attribute_set, cstr!("[]="), set as *const _, 2);
    ffi::rb_define_method(
        attribute_set,
        cstr!("values_before_type_cast"),
        values_before_type_cast as *const _,
        0,
    );
    ffi::rb_define_method(attribute_set, cstr!("to_hash"), to_hash as *const _, 0);
    ffi::rb_define_method(attribute_set, cstr!("to_h"), to_hash as *const _, 0);
    ffi::rb_define_method(attribute_set, cstr!("key?"), key_eh as *const _, 1);
    ffi::rb_define_method(attribute_set, cstr!("keys"), keys as *const _, 0);
    ffi::rb_define_method(
        attribute_set,
        cstr!("fetch_value"),
        fetch_value as *const _,
        1,
    );
    ffi::rb_define_method(
        attribute_set,
        cstr!("write_from_database"),
        write_from_database as *const _,
        2,
    );
    ffi::rb_define_method(
        attribute_set,
        cstr!("write_from_user"),
        write_from_user as *const _,
        2,
    );
    ffi::rb_define_method(attribute_set, cstr!("deep_dup"), deep_dup as *const _, 0);
    ffi::rb_define_method(
        attribute_set,
        cstr!("initialize_copy"),
        initialize_copy as *const _,
        1,
    );
    ffi::rb_define_method(attribute_set, cstr!("accessed"), accessed as *const _, 0);
    ffi::rb_define_method(attribute_set, cstr!("map"), map as *const _, 0);
    ffi::rb_define_method(attribute_set, cstr!("=="), equals as *const _, 1);
}

extern "C" fn get(this: ffi::VALUE, key: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = string_or_symbol_to_id(key);
    this.get(key)
        .map(IntoRuby::as_ruby)
        .unwrap_or(unsafe { ffi::Qnil })
}

extern "C" fn set(this: ffi::VALUE, key: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let attr = unsafe { get_struct::<Attribute>(value) };
    let key = string_or_symbol_to_id(key);
    this.set(key, attr.clone());
    unsafe { ffi::Qnil }
}

extern "C" fn values_before_type_cast(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    this.values_before_type_cast()
}

extern "C" fn to_hash(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    this.to_hash()
}

extern "C" fn key_eh(this: ffi::VALUE, key: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = string_or_symbol_to_id(key);
    to_ruby_bool(this.has_key(key))
}

extern "C" fn keys(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    this.keys()
}

extern "C" fn fetch_value(this: ffi::VALUE, key: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = string_or_symbol_to_id(key);
    this.fetch_value(key).unwrap_or(unsafe { ffi::Qnil })
}

extern "C" fn write_from_database(
    this: ffi::VALUE,
    key: ffi::VALUE,
    value: ffi::VALUE,
) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = string_or_symbol_to_id(key);
    this.write_from_database(key, value);
    unsafe { ffi::Qnil }
}

extern "C" fn write_from_user(this: ffi::VALUE, key: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    let key = string_or_symbol_to_id(key);
    this.write_from_user(key, value);
    unsafe { ffi::Qnil }
}

extern "C" fn deep_dup(this_ptr: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this_ptr) };
    this.deep_dup().into_ruby()
}

extern "C" fn initialize_copy(this_ptr: ffi::VALUE, other: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this_ptr) };
    let other = unsafe { get_struct::<AttributeSet>(other) };
    this.clone_from(other);
    this_ptr
}

extern "C" fn accessed(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    this.accessed()
}

extern "C" fn map(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<AttributeSet>(this) };
    this.map(|attr| unsafe {
        let new_attr = ffi::rb_yield(attr.as_ruby());
        get_struct::<Attribute>(new_attr).clone()
    }).into_ruby()
}

extern "C" fn equals(this: ffi::VALUE, other: ffi::VALUE) -> ffi::VALUE {
    unsafe {
        if !ffi::RB_TYPE_P(other, ffi::T_DATA) {
            return ffi::Qfalse;
        }
        if ffi::rb_obj_class(other) != AttributeSet::class() {
            return ffi::Qfalse;
        }

        let this = get_struct::<AttributeSet>(this);
        let other = get_struct::<AttributeSet>(other);
        to_ruby_bool(this == other)
    }
}
