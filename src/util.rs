use ffi;

pub unsafe fn get_struct<'a, T>(ptr: ffi::VALUE) -> &'a mut T {
    (ffi::Data_Get_Struct_Value(ptr) as *mut T)
        .as_mut()
        .unwrap_or_else(|| {
            ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Expected a T_DATA"))
        })
}
