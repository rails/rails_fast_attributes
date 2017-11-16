use ffi;
use into_ruby::*;
use super::{Attribute, MaybeProc, Source};
use util::*;

impl IntoRuby for Attribute {
    unsafe fn class() -> ffi::VALUE {
        ATTRIBUTE.unwrap()
    }

    unsafe fn mark(&self) {
        match *self {
            Attribute::Populated {
                name,
                ref raw_value,
                ty,
                ref source,
                value,
            } => {
                ffi::rb_gc_mark(name);
                raw_value.mark();
                ffi::rb_gc_mark(ty);
                if let Source::FromUser(ref orig) = *source {
                    orig.mark();
                }
                if let Some(value) = value {
                    ffi::rb_gc_mark(value);
                }
            }
            Attribute::Uninitialized { name, ty } => {
                ffi::rb_gc_mark(name);
                ffi::rb_gc_mark(ty);
            }
        }
    }
}

impl MaybeProc {
    unsafe fn mark(&self) {
        use self::MaybeProc::*;

        match *self {
            NotProc(value) => ffi::rb_gc_mark(value),
            Proc { block, ref memo } => {
                ffi::rb_gc_mark(block);
                if let Some(memo) = memo.get() {
                    ffi::rb_gc_mark(memo);
                }
            }
        }
    }
}

static mut ATTRIBUTE: Option<ffi::VALUE> = None;

pub unsafe fn init() {
    let attribute = ffi::rb_define_class_under(::module(), cstr!("Attribute"), ffi::rb_cObject);
    ATTRIBUTE = Some(attribute);

    ffi::rb_define_alloc_func(attribute, Some(Attribute::allocate));
    ffi::rb_define_singleton_method(
        attribute,
        cstr!("from_database"),
        from_database as *const _,
        3,
    );
    ffi::rb_define_singleton_method(attribute, cstr!("from_user"), from_user as *const _, 4);
    ffi::rb_define_singleton_method(
        attribute,
        cstr!("with_cast_value"),
        from_cast_value as *const _,
        3,
    );
    ffi::rb_define_singleton_method(
        attribute,
        cstr!("uninitialized"),
        uninitialized as *const _,
        2,
    );

    ffi::rb_define_method(
        attribute,
        cstr!("value_before_type_cast"),
        value_before_type_cast as *const _,
        0,
    );
    ffi::rb_define_method(attribute, cstr!("name"), name as *const _, 0);
    ffi::rb_define_method(attribute, cstr!("type"), ty as *const _, 0);
    ffi::rb_define_method(attribute, cstr!("value"), value as *const _, 0);
    ffi::rb_define_method(
        attribute,
        cstr!("original_value"),
        original_value as *const _,
        0,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("value_for_database"),
        value_for_database as *const _,
        0,
    );
    ffi::rb_define_method(attribute, cstr!("changed?"), changed_eh as *const _, 0);
    ffi::rb_define_method(
        attribute,
        cstr!("changed_in_place?"),
        changed_in_place_eh as *const _,
        0,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("forgetting_assignment"),
        forgetting_assignment as *const _,
        0,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("with_value_from_user"),
        with_value_from_user as *const _,
        1,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("with_value_from_database"),
        with_value_from_database as *const _,
        1,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("with_cast_value"),
        with_cast_value as *const _,
        1,
    );
    ffi::rb_define_method(attribute, cstr!("with_type"), with_type as *const _, 1);
    ffi::rb_define_method(
        attribute,
        cstr!("initialized?"),
        initialized_eh as *const _,
        0,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("came_from_user?"),
        came_from_user as *const _,
        0,
    );
    ffi::rb_define_method(
        attribute,
        cstr!("has_been_read?"),
        has_been_read as *const _,
        0,
    );
    ffi::rb_define_method(attribute, cstr!("=="), equals as *const _, 1);
    ffi::rb_define_method(
        attribute,
        cstr!("initialize_dup"),
        initialize_dup as *const _,
        1,
    );
    ffi::rb_define_method(attribute, cstr!("_dump_data"), dump_data as *const _, 0);
    ffi::rb_define_method(attribute, cstr!("_load_data"), load_data as *const _, 1);
}

fn from_value(value: ffi::VALUE) -> Attribute {
    unsafe {
        if ffi::rb_obj_class(value) == Attribute::class() {
            get_struct::<Attribute>(value).clone()
        } else {
            ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Expected an `Attribute`"))
        }
    }
}

extern "C" fn from_database(
    _class: ffi::VALUE,
    name: ffi::VALUE,
    value: ffi::VALUE,
    ty: ffi::VALUE,
) -> ffi::VALUE {
    Attribute::from_database(name, value, ty).into_ruby()
}

extern "C" fn from_user(
    _class: ffi::VALUE,
    name: ffi::VALUE,
    value: ffi::VALUE,
    ty: ffi::VALUE,
    original_attribute: ffi::VALUE,
) -> ffi::VALUE {
    let original_attribute = from_value(original_attribute);
    Attribute::from_user(name, value, ty, original_attribute).into_ruby()
}

extern "C" fn from_cast_value(
    _class: ffi::VALUE,
    name: ffi::VALUE,
    value: ffi::VALUE,
    ty: ffi::VALUE,
) -> ffi::VALUE {
    Attribute::from_cast_value(name, value, ty).into_ruby()
}

extern "C" fn uninitialized(_class: ffi::VALUE, name: ffi::VALUE, ty: ffi::VALUE) -> ffi::VALUE {
    Attribute::uninitialized(name, ty).into_ruby()
}

extern "C" fn value_before_type_cast(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.value_before_type_cast()
}

extern "C" fn name(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.name()
}

extern "C" fn ty(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.ty()
}

extern "C" fn value(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.value()
}

extern "C" fn original_value(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.original_value()
}

extern "C" fn value_for_database(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.value_for_database()
}

extern "C" fn changed_eh(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    to_ruby_bool(this.is_changed())
}

extern "C" fn changed_in_place_eh(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    to_ruby_bool(this.is_changed_in_place())
}

extern "C" fn forgetting_assignment(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.forgetting_assignment().into_ruby()
}

extern "C" fn with_value_from_user(this: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.with_value_from_user(value).into_ruby()
}

extern "C" fn with_value_from_database(this: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.with_value_from_database(value).into_ruby()
}

extern "C" fn with_cast_value(this: ffi::VALUE, value: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.with_cast_value(value).into_ruby()
}

extern "C" fn with_type(this: ffi::VALUE, ty: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    this.with_type(ty).into_ruby()
}

extern "C" fn initialized_eh(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    to_ruby_bool(this.is_initialized())
}

extern "C" fn came_from_user(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    to_ruby_bool(this.came_from_user())
}

extern "C" fn has_been_read(this: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    to_ruby_bool(this.has_been_read())
}

extern "C" fn equals(this: ffi::VALUE, other: ffi::VALUE) -> ffi::VALUE {
    unsafe {
        if !ffi::RB_TYPE_P(other, ffi::T_DATA) {
            return ffi::Qfalse;
        }
        if ffi::rb_obj_class(other) != Attribute::class() {
            return ffi::Qfalse;
        }

        let this = get_struct::<Attribute>(this);
        let other = get_struct::<Attribute>(other);
        to_ruby_bool(this == other)
    }
}

extern "C" fn initialize_dup(this: ffi::VALUE, other: ffi::VALUE) -> ffi::VALUE {
    let this = unsafe { get_struct::<Attribute>(this) };
    let other = unsafe { get_struct::<Attribute>(other) };
    this.initialize_dup(other);
    unsafe { ffi::Qnil }
}

extern "C" fn dump_data(this: ffi::VALUE) -> ffi::VALUE {
    use self::Attribute::*;
    let this = unsafe { get_struct::<Attribute>(this) };

    return match *this {
        Populated {
            name,
            ref raw_value,
            ty,
            ref source,
            value: _value,
        } => to_ruby_array(4, vec![name, ty, raw_value.value(), dump_source(source)]),
        Uninitialized { name, ty } => to_ruby_array(2, vec![name, ty]),
    };

    fn dump_source(source: &'static Source) -> ffi::VALUE {
        use self::Source::*;
        match *source {
            FromUser(ref orig) => orig.as_ruby(),
            FromDatabase => unsafe { ffi::I322NUM(2) },
            PreCast => unsafe { ffi::I322NUM(3) },
        }
    }
}

extern "C" fn load_data(this: ffi::VALUE, data: ffi::VALUE) -> ffi::VALUE {
    use self::Attribute::*;
    use self::Source::*;
    use self::MaybeProc::*;

    fn error() -> ! {
        unsafe { ffi::rb_raise(ffi::rb_eRuntimeError, cstr!("Unrecognized attribute")) };
    }

    unsafe {
        let this = get_struct::<Attribute>(this);
        let name = ffi::rb_ary_entry(data, 0);
        let ty = ffi::rb_ary_entry(data, 1);
        let raw_value = NotProc(ffi::rb_ary_entry(data, 2));
        let source = ffi::rb_ary_entry(data, 3);

        if ffi::RB_NIL_P(source) {
            *this = Uninitialized { name, ty };
        } else if ffi::RB_TYPE_P(source, ffi::T_DATA) {
            let attr = get_struct::<Attribute>(source).clone();
            let source = FromUser(Box::new(attr));
            *this = Populated {
                name,
                ty,
                raw_value,
                source,
                value: None,
            };
        } else if ffi::RB_TYPE_P(source, ffi::T_FIXNUM) {
            let source = match ffi::NUM2I32(source) {
                2 => FromDatabase,
                3 => PreCast,
                _ => error(),
            };
            *this = Populated {
                name,
                ty,
                raw_value,
                source,
                value: None,
            };
        } else {
            error();
        }

        ffi::Qnil
    }
}
