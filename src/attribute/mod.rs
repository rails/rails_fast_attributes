use std::cell::Cell;

use ffi;
use into_ruby::IntoRuby;

mod ruby_glue;

#[derive(Clone, Eq)]
pub enum Attribute {
    Populated {
        name: ffi::VALUE,
        raw_value: MaybeProc,
        ty: ffi::VALUE,
        source: Source,
        value: Cell<Option<ffi::VALUE>>,
    },
    Uninitialized {
        name: ffi::VALUE,
        ty: ffi::VALUE,
    },
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
    UserProvidedDefault(Option<Box<Attribute>>),
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

impl Drop for Attribute {
    fn drop(&mut self) {

        match *self {
            Attribute::Populated {
                ref mut value,
                ref mut raw_value,
                ..
            } =>  {
                value.set(Some(unsafe{ffi::Qnil}));
                match *raw_value {
                    MaybeProc::Proc {
                        ref mut memo,
                        ..
                    } => memo.set(Some(unsafe{ffi::Qnil})),
                    MaybeProc::NotProc(ref mut val) => *val = unsafe{ffi::Qnil}
                }
            },
            Attribute::Uninitialized{
                ref mut name,
                ..
            } => *name = unsafe{ffi::Qnil}

        }
    }
}

impl Attribute {
    pub fn from_database(name: ffi::VALUE, raw_value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value: MaybeProc::NotProc(raw_value),
            ty,
            source: Source::FromDatabase,
            value: Cell::new(None),
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
            value: Cell::new(None),
        }
    }

    fn from_cast_value(name: ffi::VALUE, value: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Populated {
            name,
            raw_value: MaybeProc::NotProc(value),
            ty,
            source: Source::PreCast,
            value: Cell::new(None),
        }
    }

    pub fn uninitialized(name: ffi::VALUE, ty: ffi::VALUE) -> Self {
        Attribute::Uninitialized { name, ty }
    }

    pub fn user_provided_default(
        name: ffi::VALUE,
        raw_value: MaybeProc,
        ty: ffi::VALUE,
        original_attribute: Option<Attribute>,
    ) -> Self {
        Attribute::Populated {
            name,
            raw_value,
            ty,
            source: Source::UserProvidedDefault(original_attribute.map(Box::new)),
            value: Cell::new(None),
        }
    }

    pub fn value_before_type_cast(&self) -> ffi::VALUE {
        if let Attribute::Populated { ref raw_value, .. } = *self {
            raw_value.value()
        } else {
            unsafe { ffi::Qnil }
        }
    }

    pub fn value(&self) -> ffi::VALUE {
        use self::Attribute::*;

        unsafe {
            match *self {
                Populated {
                    ref value,
                    ref source,
                    ty,
                    ref raw_value,
                    ..
                } => {
                    if value.get().is_none() {
                        value.set(Some(cast_value(source, ty, raw_value.value())));
                    }
                    value.get().unwrap()
                }
                Uninitialized { name, .. } => if ffi::rb_block_given_p() {
                    ffi::rb_yield(name)
                } else {
                    ffi::Qnil
                },
            }
        }
    }

    fn value_for_database(&self) -> ffi::VALUE {
        let value = self.value();
        let ty = self.ty();
        unsafe { ffi::rb_funcall(ty, id!("serialize"), 1, value) }
    }

    fn is_changed(&self) -> bool {
        self.is_changed_from_assignment() || self.is_changed_in_place()
    }

    fn is_changed_in_place(&self) -> bool {
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

    fn forgetting_assignment(&self) -> Self {
        match *self {
            Attribute::Populated { .. } => {
                let value_for_database = self.value_for_database();
                self.with_value_from_database(value_for_database)
            }
            Attribute::Uninitialized { .. } => self.clone(),
        }
    }

    pub fn with_value_from_user(self, value: ffi::VALUE) -> Self {
        let ty = self.ty();
        unsafe {
            ffi::rb_funcall(ty, id!("assert_valid_value"), 1, value);
        }
        Self::from_user(self.name(), value, ty, self)
    }

    pub fn with_value_from_database(&self, value: ffi::VALUE) -> Self {
        Self::from_database(self.name(), value, self.ty())
    }

    pub fn with_cast_value(&self, value: ffi::VALUE) -> Self {
        Self::from_cast_value(self.name(), value, self.ty())
    }

    fn with_type(&self, ty: ffi::VALUE) -> Self {
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
                    value: Cell::new(None),
                },
                Uninitialized { name, .. } => Uninitialized { name, ty },
            }
        }
    }

    pub fn came_from_user(&self) -> bool {
        if let Attribute::Populated {
            source: Source::FromUser(_),
            ref raw_value,
            ty,
            ..
        } = *self
        {
            unsafe {
                !ffi::RTEST(ffi::rb_funcall(
                    ty,
                    id!("value_constructed_by_mass_assignment?"),
                    1,
                    raw_value.value(),
                ))
            }
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
        if let Attribute::Populated { ref value, .. } = *self {
            value.get().is_some()
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
                ref value,
            } => Populated {
                name,
                raw_value: raw_value.clone(),
                ty,
                source: source.clone(),
                value: Cell::new(value.get().map(|v| unsafe { ffi::rb_obj_dup(v) })),
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

    fn is_changed_from_assignment(&self) -> bool {
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
        use self::Attribute::*;
        use self::Source::*;

        match *self {
            Populated { ref source, .. } => match *source {
                FromUser(_) | UserProvidedDefault(Some(_)) => true,
                _ => false,
            },
            _ => false,
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
                FromDatabase | UserProvidedDefault(None) => {
                    cast_value(source, ty, raw_value.value())
                }
                PreCast => raw_value.value(),
                UserProvidedDefault(Some(ref orig)) => orig.original_value(),
            },
            Uninitialized { .. } => unsafe {
                ffi::rb_const_get(Self::class(), id!("UNINITIALIZED_ORIGINAL_VALUE"))
            },
        }
    }

    fn original_value_for_database(&self) -> ffi::VALUE {
        use self::Attribute::*;
        use self::Source::*;
        match *self {
            Populated {
                source: FromUser(ref orig),
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
            Populated {
                source: UserProvidedDefault(Some(ref orig)),
                ..
            } => orig.original_value_for_database(),
            Populated {
                source: UserProvidedDefault(None),
                ty,
                ..
            } => {
                let value = self.original_value();
                unsafe { ffi::rb_funcall(ty, id!("serialize"), 1, value) }
            }
            Uninitialized { .. } => unsafe { ffi::Qnil },
        }
    }

    fn original_attribute(&self) -> Option<&Attribute> {
        use self::Attribute::*;
        use self::Source::*;

        match *self {
            Populated {
                source: FromUser(ref orig),
                ..
            } => Some(orig),
            Populated {
                source: UserProvidedDefault(Some(ref orig)),
                ..
            } => Some(&**orig),
            _ => None,
        }
    }

    pub fn deep_dup(&self) -> Self {
        let mut result = Self::default();
        result.initialize_dup(self);
        result
    }

    pub fn without_cast_value(&self) -> Self {
        let result = self.clone();
        if let Attribute::Populated { ref value, .. } = result {
            value.set(None);
        }
        result
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
            FromUser(_) | UserProvidedDefault(_) => ffi::rb_funcall(ty, id!("cast"), 1, raw_value),
            PreCast => raw_value,
        }
    }
}

fn ruby_equals(lhs: ffi::VALUE, rhs: ffi::VALUE) -> bool {
    unsafe { ffi::RTEST(ffi::rb_funcall(lhs, id!("=="), 1, rhs)) }
}
