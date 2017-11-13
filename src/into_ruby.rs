use {ffi, libc};

pub trait IntoRuby: Sized {
    unsafe fn class() -> ffi::VALUE;
    unsafe fn mark(&self);

    extern "C" fn mark_ptr(this: *mut libc::c_void) {
        let this = this as *mut Self;
        unsafe {
            if let Some(this) = this.as_ref() {
                this.mark()
            }
        }
    }

    extern "C" fn destroy_ptr(this: *mut libc::c_void) {
        if !this.is_null() {
            let _ = unsafe { Box::from_raw(this as *mut Self) };
        }
    }

    fn as_ruby(&'static self) -> ffi::VALUE {
        unsafe {
            ffi::Data_Wrap_Struct(
                Self::class(),
                Some(Self::mark_ptr),
                None,
                self as *const _ as *mut _,
            )
        }
    }

    fn into_ruby(self) -> ffi::VALUE {
        let ptr = Box::into_raw(Box::new(self));
        unsafe {
            ffi::Data_Wrap_Struct(
                Self::class(),
                Some(Self::mark_ptr),
                Some(Self::destroy_ptr),
                ptr as *mut _,
            )
        }
    }
}

pub trait Allocate: Default + IntoRuby {
    extern "C" fn allocate(class: ffi::VALUE) -> ffi::VALUE {
        let ptr = Box::into_raw(Box::new(Self::default()));

        unsafe {
            ffi::Data_Wrap_Struct(
                class,
                Some(Self::mark_ptr),
                Some(Self::destroy_ptr),
                ptr as *mut _,
            )
        }
    }
}

impl<T> Allocate for T
where
    T: Default + IntoRuby,
{
}
