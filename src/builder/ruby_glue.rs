use {ffi, libc};
use into_ruby::{Allocate, IntoRuby};
use super::Builder;
use util::*;

impl IntoRuby for Builder {
    unsafe fn class() -> ffi::VALUE {
        BUILDER.unwrap()
    }

    unsafe fn mark(&self) {
        for (&key, value) in &self.uninitialized_attributes {
            let sym = ffi::rb_id2sym(key);
            ffi::rb_gc_mark(sym);
            value.mark();
        }
    }
}

static mut BUILDER: Option<ffi::VALUE> = None;

pub unsafe fn init() {
    let builder = ffi::rb_define_class_under(::module(), cstr!("Builder"), ffi::rb_cObject);
    BUILDER = Some(builder);

    ffi::rb_define_alloc_func(builder, Builder::allocate);

    ffi::rb_define_method(builder, cstr!("initialize"), initialize as *const _, -1);
    ffi::rb_define_method(
        builder,
        cstr!("build_from_database"),
        build_from_database as *const _,
        -1,
    );
}

extern "C" fn initialize(
    argc: libc::c_int,
    argv: *const ffi::VALUE,
    this: ffi::VALUE,
) -> ffi::VALUE {
    unsafe {
        let mut types = ffi::Qnil;
        let mut default_attributes = ffi::Qnil;
        ffi::rb_scan_args(argc, argv, cstr!("11"), &mut types, &mut default_attributes);

        let default_attributes = if ffi::RB_NIL_P(default_attributes) {
            None
        } else {
            Some(default_attributes)
        };

        let this = get_struct_mut::<Builder>(this);
        this.initialize(types, default_attributes);
    }
    this
}

extern "C" fn build_from_database(
    argc: libc::c_int,
    argv: *const ffi::VALUE,
    this: ffi::VALUE,
) -> ffi::VALUE {
    unsafe {
        let this = get_struct::<Builder>(this);
        let mut values = ffi::Qnil;
        let mut additional_types = ffi::Qnil;
        ffi::rb_scan_args(argc, argv, cstr!("02"), &mut values, &mut additional_types);

        if ffi::RB_NIL_P(values) {
            values = ffi::rb_hash_new();
        }

        let additional_types = if ffi::RB_NIL_P(additional_types) {
            None
        } else {
            Some(additional_types)
        };

        this.build_from_database(values, additional_types)
            .into_ruby()
    }
}
