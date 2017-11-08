use ffi;

mod ruby_glue;

#[derive(Clone, Eq)]
pub enum Attribute {
    Populated {
        name: ffi::VALUE,
        raw_value: ffi::VALUE,
        ty: ffi::VALUE,
        source: Source,
        value: Option<ffi::VALUE>,
    },
    Uninitialized { name: ffi::VALUE, ty: ffi::VALUE },
}

impl Default for Attribute {
    fn default() -> Self {
        let nil = unsafe { ffi::Qnil };
        Self::uninitialized(nil, nil)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Source {
    FromUser,
    FromDatabase,
}

impl Attribute {
    fn from_database(name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value,
            ty,
            source: Source::FromDatabase,
            value: None,
        }
    }

    fn from_user(name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value,
            ty,
            source: Source::FromUser,
            value: None,
        }
    }

    fn uninitialized(name: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Uninitialized { name, ty }
    }

    fn value_before_type_cast(&self) -> ffi::VALUE {
        if let Attribute::Populated { raw_value, .. } = *self {
            raw_value
        } else {
            unsafe { ffi::Qnil }
        }
    }

    fn value(&mut self) -> ffi::VALUE {
        use self::Attribute::*;
        use self::Source::*;

        unsafe {
            match *self {
                Populated {
                    ref mut value,
                    ref source,
                    ty,
                    raw_value,
                    ..
                } => {
                    if value.is_none() {
                        *value = Some(match *source {
                            FromDatabase => ffi::rb_funcall(ty, id!("deserialize"), 1, raw_value),
                            FromUser => ffi::rb_funcall(ty, id!("cast"), 1, raw_value),
                        });
                    }
                    value.unwrap()
                }
                Uninitialized { name, .. } => if ffi::rb_block_given_p() {
                    ffi::rb_yield(name)
                } else {
                    ffi::Qnil
                },
            }
        }
    }

    fn value_for_database(&mut self) -> ffi::VALUE {
        let value = self.value();
        let ty = self.ty();
        unsafe { ffi::rb_funcall(ty, id!("serialize"), 1, value) }
    }

    fn with_value_from_user(&self, value: ffi::VALUE) -> Self {
        Self::from_user(self.name(), value, self.ty())
    }

    fn with_value_from_database(&self, value: ffi::VALUE) -> Self {
        Self::from_database(self.name(), value, self.ty())
    }

    fn has_been_read(&self) -> bool {
        if let Attribute::Populated { value: Some(_), .. } = *self {
            true
        } else {
            false
        }
    }

    fn initialize_dup(&mut self, other: &Attribute) {
        use self::Attribute::*;
        *self = match *other {
            Populated {
                name,
                raw_value,
                ty,
                ref source,
                value: Some(value),
            } => Populated {
                name,
                raw_value,
                ty,
                source: source.clone(),
                value: Some(unsafe { ffi::rb_obj_dup(value) }),
            },
            _ => other.clone(),
        }
    }

    fn name(&self) -> ffi::VALUE {
        match *self {
            Attribute::Populated { name, .. } => name,
            Attribute::Uninitialized { name, .. } => name,
        }
    }

    fn ty(&self) -> ffi::VALUE {
        match *self {
            Attribute::Populated { ty, .. } => ty,
            Attribute::Uninitialized { ty, .. } => ty,
        }
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        fn ruby_equals(lhs: ffi::VALUE, rhs: ffi::VALUE) -> bool {
            unsafe { ffi::RTEST(ffi::rb_funcall(lhs, id!("=="), 1, rhs)) }
        }

        use self::Attribute::*;

        match (self, other) {
            (
                &Populated {
                    ref source,
                    name,
                    raw_value,
                    ty,
                    ..
                },
                &Populated {
                    source: ref source2,
                    name: name2,
                    raw_value: val2,
                    ty: ty2,
                    ..
                },
            ) => {
                source == source2 && ruby_equals(name, name2) && ruby_equals(raw_value, val2)
                    && ruby_equals(ty, ty2)
            }
            (
                &Uninitialized { name, ty },
                &Uninitialized {
                    name: name2,
                    ty: ty2,
                },
            ) => ruby_equals(name, name2) && ruby_equals(ty, ty2),
            _ => false,
        }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
