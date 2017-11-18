use ffi;

pub unsafe fn get_struct<'a, T>(ptr: ffi::VALUE) -> &'a T {
    (ffi::Data_Get_Struct_Value(ptr) as *mut T)
        .as_ref()
        .unwrap_or_else(|| {
            ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Expected a T_DATA"))
        })
}

pub unsafe fn get_struct_mut<'a, T>(ptr: ffi::VALUE) -> &'a mut T {
    if ffi::OBJ_FROZEN(ptr) {
        ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Can't modify frozen object"));
    }

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

pub fn to_ruby_array<T>(capactiy: usize, iter: T) -> ffi::VALUE
where
    T: IntoIterator<Item = ffi::VALUE>,
{
    let result = unsafe { ffi::rb_ary_new_capa(capactiy as isize) };
    for item in iter {
        unsafe { ffi::rb_ary_push(result, item) };
    }
    result
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
