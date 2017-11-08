use ffi;

mod ruby_glue;

pub struct Attribute {
    raw_value: ffi::VALUE,
    ty: ffi::VALUE,
}

impl Attribute {
    fn from_database(_name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute { raw_value, ty }
    }

    fn value(&self) -> ffi::VALUE {
        unsafe { ffi::rb_funcall(self.ty, id!("deserialize"), 1, self.raw_value) }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
