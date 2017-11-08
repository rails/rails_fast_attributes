use ffi;

mod ruby_glue;

#[derive(Clone)]
pub struct Attribute {
    raw_value: ffi::VALUE,
    ty: ffi::VALUE,
    source: Source,
    value: Option<ffi::VALUE>,
}

impl Default for Attribute {
    fn default() -> Self {
        let nil = unsafe { ffi::Qnil };
        Self::from_database(nil, nil, nil)
    }
}

#[derive(Clone)]
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

    fn initialize_dup(&mut self, other: &Attribute) {
        *self = other.clone();
        if let Some(value) = self.value {
            self.value = Some(unsafe { ffi::rb_obj_dup(value) });
        }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
