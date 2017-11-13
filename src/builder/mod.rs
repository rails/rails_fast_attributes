use std::collections::HashMap;

use attribute::Attribute;
use attribute_set::AttributeSet;
use {ffi, libc};

mod ruby_glue;

#[derive(Default, Clone)]
pub struct Builder {
    uninitialized_attributes: HashMap<ffi::ID, Attribute>,
}

impl Builder {
    unsafe fn initialize(&mut self, types: ffi::VALUE) {
        if !ffi::RB_TYPE_P(types, ffi::T_HASH) {
            ffi::rb_raise(ffi::rb_eTypeError, cstr!("Expected a Hash"));
        }

        self.uninitialized_attributes
            .reserve(ffi::RHASH_SIZE(types) as usize);
        ffi::rb_hash_foreach(
            types,
            Some(push_uninitialized_value),
            &mut self.uninitialized_attributes as *mut _ as *mut _,
        );
    }

    fn build_from_database(&self, values: ffi::VALUE, additional_types: Option<ffi::VALUE>) -> AttributeSet {
        let mut attributes = self.uninitialized_attributes.clone();

        unsafe {
            if let Some(types) = additional_types {
                ffi::rb_hash_foreach(
                    types,
                    Some(push_uninitialized_value),
                    &mut attributes as *mut _ as *mut _,
                );
            }

            ffi::rb_hash_foreach(
                values,
                Some(push_value),
                &mut attributes as *mut _ as *mut _,
            );
        }

        AttributeSet::new(attributes)
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}

extern "C" fn push_uninitialized_value(
    key: ffi::VALUE,
    value: ffi::VALUE,
    hash_ptr: *mut libc::c_void,
) -> ffi::st_retval {
    let hash_ptr = hash_ptr as *mut HashMap<ffi::ID, Attribute>;
    let hash = unsafe { hash_ptr.as_mut().unwrap() };

    let id = unsafe { ffi::rb_sym2id(key) };
    let attribute = Attribute::uninitialized(key, value);

    hash.insert(id, attribute);

    ffi::st_retval::ST_CONTINUE
}

extern "C" fn push_value(
    key: ffi::VALUE,
    value: ffi::VALUE,
    data_ptr: *mut libc::c_void,
) -> ffi::st_retval {
    let data_ptr = data_ptr as *mut HashMap<ffi::ID, Attribute>;
    let hash = unsafe { data_ptr.as_mut().unwrap() };

    let id = unsafe { ffi::rb_sym2id(key) };

    let new_attr = hash[&id].with_value_from_database(value);
    hash.insert(id, new_attr);

    ffi::st_retval::ST_CONTINUE
}
