use ffi;

mod ruby_glue;

pub struct Attribute {
    raw_value: ffi::VALUE,
    ty: ffi::VALUE,
    source: Source,
    value: Option<ffi::VALUE>,
}

pub enum Source {
    FromUser,
    FromDatabase,
}

impl Attribute {
    fn from_database(_name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute {
            raw_value,
            ty,
            source: Source::FromDatabase,
            value: None,
        }
    }

    fn from_user(_name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute {
            raw_value,
            ty,
            source: Source::FromUser,
            value: None,
        }
    }

    fn value_before_type_cast(&self) -> ffi::VALUE {
        self.raw_value
    }

    fn value(&mut self) -> ffi::VALUE {
        use self::Source::*;

        if self.value.is_none() {
            self.value = Some(match self.source {
                FromDatabase => unsafe {
                    ffi::rb_funcall(self.ty, id!("deserialize"), 1, self.raw_value)
                },
                FromUser => unsafe { ffi::rb_funcall(self.ty, id!("cast"), 1, self.raw_value) },
            });
        }
        self.value.unwrap()
    }

    fn value_for_database(&mut self) -> ffi::VALUE {
        let value = self.value();
        unsafe {
            ffi::rb_funcall(self.ty, id!("serialize"), 1, value)
        }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
