use ffi;

mod ruby_glue;

pub struct Attribute {
    raw_value: ffi::VALUE,
    ty: ffi::VALUE,
    source: Source,
}

pub enum Source {
    FromUser,
    FromDatabase,
}

impl Attribute {
    fn from_database(_name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute { raw_value, ty, source: Source::FromDatabase }
    }

    fn from_user(_name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute { raw_value, ty, source: Source::FromUser }
    }

    fn value(&self) -> ffi::VALUE {
        use self::Source::*;

        match self.source {
            FromDatabase => unsafe { ffi::rb_funcall(self.ty, id!("deserialize"), 1, self.raw_value) }
            FromUser => unsafe { ffi::rb_funcall(self.ty, id!("cast"), 1, self.raw_value) }
        }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
