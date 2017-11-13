use ffi;

pub unsafe fn get_struct<'a, T>(ptr: ffi::VALUE) -> &'a mut T {
    (ffi::Data_Get_Struct_Value(ptr) as *mut T)
        .as_mut()
        .unwrap_or_else(|| {
            ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Expected a T_DATA"))
        })
}

pub fn to_ruby_bool(test: bool) -> ffi::VALUE {
    if test {
        unsafe { ffi::Qtrue }
    } else {
        unsafe { ffi::Qfalse }
    }
}


pub fn string_or_symbol_to_id(sym_or_string: ffi::VALUE) -> ffi::ID {
    unsafe {
        if ffi::RB_TYPE_P(sym_or_string, ffi::T_STRING) {
            ffi::rb_intern_str(sym_or_string)
        } else {
            ffi::rb_sym2id(sym_or_string)
        }
    }
}
