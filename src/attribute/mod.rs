use std::cell::Cell;

use ffi;

mod ruby_glue;

#[derive(Clone, Eq)]
pub enum Attribute {
    Populated {
        name: ffi::VALUE,
        raw_value: MaybeProc,
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
    FromUser(Box<Attribute>),
    FromDatabase,
    PreCast,
}

#[derive(Clone, Eq)]
/// Represents either a Ruby value, or a block which needs to be called and
/// memoized to get the Ruby value.
pub enum MaybeProc {
    NotProc(ffi::VALUE),
    Proc {
        block: ffi::VALUE,
        memo: Cell<Option<ffi::VALUE>>,
    },
}

impl Attribute {
    pub fn from_database(name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value: MaybeProc::NotProc(raw_value),
            ty,
            source: Source::FromDatabase,
            value: None,
        }
    }

    fn from_user(
        name: ffi::VALUE,
        raw_value: ffi::VALUE,
        ty: ffi::VALUE,
        original_attribute: Attribute,
    ) -> Self {
        Attribute::Populated {
            name,
            raw_value: MaybeProc::NotProc(raw_value),
            ty,
            source: Source::FromUser(Box::new(original_attribute)),
            value: None,
        }
    }

    fn from_cast_value(name: ffi::VALUE, value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value: MaybeProc::NotProc(value),
            ty,
            source: Source::PreCast,
            value: Some(value),
        }
    }

    pub fn uninitialized(name: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Uninitialized { name, ty }
    }

    pub fn value_before_type_cast(&self) -> ffi::VALUE {
        if let Attribute::Populated { ref raw_value, .. } = *self {
            raw_value.value()
        } else {
            unsafe { ffi::Qnil }
        }
    }

    pub fn value(&mut self) -> ffi::VALUE {
        use self::Attribute::*;

        unsafe {
            match *self {
                Populated {
                    ref mut value,
                    ref source,
                    ty,
                    ref raw_value,
                    ..
                } => {
                    if value.is_none() {
                        *value = Some(cast_value(source, ty, raw_value.value()));
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

    fn is_changed(&mut self) -> bool {
        self.is_changed_from_assignment() || self.is_changed_in_place()
    }

    fn is_changed_in_place(&mut self) -> bool {
        if !self.has_been_read() {
            return false;
        }

        unsafe {
            let orig = self.original_value_for_database();
            let value = self.value();
            ffi::RTEST(ffi::rb_funcall(
                self.ty(),
                id!("changed_in_place?"),
                2,
                orig,
                value,
            ))
        }
    }

    fn forgetting_assignment(&mut self) -> Self {
        let value_for_database = self.value_for_database();
        self.with_value_from_database(value_for_database)
    }

    pub fn with_value_from_user(&self, value: ffi::VALUE) -> Self {
        let ty = self.ty();
        unsafe {
            ffi::rb_funcall(ty, id!("assert_valid_value"), 1, value);
        }
        Self::from_user(self.name(), value, ty, self.clone())
    }

    pub fn with_value_from_database(&self, value: ffi::VALUE) -> Self {
        Self::from_database(self.name(), value, self.ty())
    }

    pub fn with_cast_value(&self, value: ffi::VALUE) -> Self {
        Self::from_cast_value(self.name(), value, self.ty())
    }

    fn with_type(&mut self, ty: ffi::VALUE) -> Self {
        use self::Attribute::*;

        if self.is_changed_in_place() {
            Self::from_user(self.name(), self.value(), ty, self.clone())
        } else {
            match *self {
                Populated {
                    name,
                    ref raw_value,
                    ref source,
                    ..
                } => Populated {
                    name,
                    raw_value: raw_value.clone(),
                    source: source.clone(),
                    ty,
                    value: None,
                },
                Uninitialized { name, .. } => Uninitialized { name, ty },
            }
        }
    }

    pub fn came_from_user(&self) -> bool {
        if let Attribute::Populated {
            source: Source::FromUser(_),
            ..
        } = *self
        {
            true
        } else {
            false
        }
    }

    pub fn is_initialized(&self) -> bool {
        if let Attribute::Uninitialized { .. } = *self {
            false
        } else {
            true
        }
    }

    pub fn has_been_read(&self) -> bool {
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
                ref raw_value,
                ty,
                ref source,
                value: Some(value),
            } => Populated {
                name,
                raw_value: raw_value.clone(),
                ty,
                source: source.clone(),
                value: Some(unsafe { ffi::rb_obj_dup(value) }),
            },
            _ => other.clone(),
        }
    }

    pub fn name(&self) -> ffi::VALUE {
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

    fn is_changed_from_assignment(&mut self) -> bool {
        if !self.has_been_assigned() {
            return false;
        }

        let ty = self.ty();
        let orig = self.original_value();
        let value = self.value();
        let raw_value = self.value_before_type_cast();

        unsafe {
            ffi::RTEST(ffi::rb_funcall(
                ty,
                id!("changed?"),
                3,
                orig,
                value,
                raw_value,
            ))
        }
    }

    fn has_been_assigned(&self) -> bool {
        if let Attribute::Populated {
            source: Source::FromUser(_),
            ..
        } = *self
        {
            true
        } else {
            false
        }
    }

    fn original_value(&self) -> ffi::VALUE {
        use self::Attribute::*;
        use self::Source::*;
        match *self {
            Populated {
                ref source,
                ty,
                ref raw_value,
                ..
            } => match *source {
                FromUser(ref orig) => orig.original_value(),
                FromDatabase => cast_value(source, ty, raw_value.value()),
                PreCast => raw_value.value(),
            },
            Uninitialized { .. } => unsafe { ffi::Qnil }, // FIXME: This is a marker object in Ruby
        }
    }

    fn original_value_for_database(&mut self) -> ffi::VALUE {
        use self::Attribute::*;
        use self::Source::*;
        match *self {
            Populated {
                source: FromUser(ref mut orig),
                ..
            } => orig.original_value_for_database(),
            Populated {
                source: FromDatabase,
                ref raw_value,
                ..
            } => raw_value.value(),
            Populated {
                source: PreCast, ..
            } => self.value_for_database(),
            Uninitialized { .. } => unsafe { ffi::Qnil },
        }
    }

    pub fn deep_dup(&self) -> Self {
        use self::Attribute::Populated;

        match *self {
            Populated {
                ref source,
                name,
                ref raw_value,
                ty,
                value: Some(value),
            } => Populated {
                source: source.clone(),
                name,
                raw_value: raw_value.clone(),
                ty,
                value: Some(unsafe { ffi::rb_obj_dup(value) }),
            },
            _ => self.clone(),
        }
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        use self::Attribute::*;

        match (self, other) {
            (
                &Populated {
                    ref source,
                    name,
                    ref raw_value,
                    ty,
                    ..
                },
                &Populated {
                    source: ref source2,
                    name: name2,
                    raw_value: ref val2,
                    ty: ty2,
                    ..
                },
            ) => {
                source == source2 && ruby_equals(name, name2) && raw_value == val2
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

impl MaybeProc {
    fn value(&self) -> ffi::VALUE {
        use self::MaybeProc::*;

        match *self {
            NotProc(value) => value,
            Proc { block, ref memo } => {
                if memo.get().is_none() {
                    let value = unsafe { ffi::rb_funcall(block, id!("call"), 0) };
                    memo.set(Some(value));
                }
                memo.get().unwrap()
            }
        }
    }
}

impl PartialEq for MaybeProc {
    fn eq(&self, other: &Self) -> bool {
        ruby_equals(self.value(), other.value())
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}

fn cast_value(source: &Source, ty: ffi::VALUE, raw_value: ffi::VALUE) -> ffi::VALUE {
    use self::Source::*;
    unsafe {
        match *source {
            FromDatabase => ffi::rb_funcall(ty, id!("deserialize"), 1, raw_value),
            FromUser(_) => ffi::rb_funcall(ty, id!("cast"), 1, raw_value),
            PreCast => raw_value,
        }
    }
}

fn ruby_equals(lhs: ffi::VALUE, rhs: ffi::VALUE) -> bool {
    unsafe { ffi::RTEST(ffi::rb_funcall(lhs, id!("=="), 1, rhs)) }
}
