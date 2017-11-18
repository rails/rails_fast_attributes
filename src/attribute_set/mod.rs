use ordermap::OrderMap;
use std::ffi::CString;

use attribute::Attribute;
use ffi;
use util::to_ruby_array;

mod ruby_glue;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct AttributeSet {
    attributes: OrderMap<ffi::ID, Attribute>,
}

impl AttributeSet {
    pub fn new(attributes: OrderMap<ffi::ID, Attribute>) -> Self {
        Self { attributes }
    }

    fn each_value<'a, F: Fn(&'a Attribute)>(&'a self, f: F) {
        for attr in self.attributes.values() {
            f(attr)
        }
    }

    fn get(&self, key: ffi::ID) -> Option<&Attribute> {
        self.attributes.get(&key)
    }

    fn set(&mut self, key: ffi::ID, attr: Attribute) {
        self.attributes.insert(key, attr);
    }

    fn values_before_type_cast(&self) -> ffi::VALUE {
        let result = unsafe { ffi::rb_hash_new() };
        for attr in self.attributes.values() {
            let name = attr.name();
            let value = attr.value_before_type_cast();
            unsafe { ffi::rb_hash_aset(result, name, value) };
        }
        result
    }

    fn to_hash(&self) -> ffi::VALUE {
        let result = unsafe { ffi::rb_hash_new() };
        let attributes = self.attributes
            .values()
            .filter(|attr| attr.is_initialized());
        for attr in attributes {
            unsafe { ffi::rb_hash_aset(result, attr.name(), attr.value()) };
        }
        result
    }

    fn has_key(&self, key: ffi::ID) -> bool {
        self.get(key)
            .map(Attribute::is_initialized)
            .unwrap_or(false)
    }

    fn keys(&self) -> ffi::VALUE {
        let keys = self.attributes
            .values()
            .filter(|a| a.is_initialized())
            .map(Attribute::name);

        to_ruby_array(self.attributes.len(), keys)
    }

    fn fetch_value(&self, key: ffi::ID) -> Option<ffi::VALUE> {
        self.get(key).map(Attribute::value)
    }

    fn write_from_database(&mut self, key: ffi::ID, value: ffi::VALUE) {
        let new_attr = self.get(key).map(|a| a.with_value_from_database(value));
        if let Some(attr) = new_attr {
            self.attributes.insert(key, attr);
        } else {
            missing_attribute(key)
        }
    }

    fn write_from_user(&mut self, key: ffi::ID, value: ffi::VALUE) {
        use ordermap::Entry::*;
        use std::mem::swap;

        match self.attributes.entry(key) {
            Vacant(_) => missing_attribute(key),
            Occupied(mut entry) => {
                // `with_value_from_user` requires ownership, so we need to
                // temporarily pull the value out of the map. We don't want
                // to use `remove`, because that would mess with the order.
                let nil = unsafe { ffi::Qnil };
                let mut tmp = Attribute::uninitialized(nil, nil);
                swap(entry.get_mut(), &mut tmp);
                tmp = tmp.with_value_from_user(value);
                swap(entry.get_mut(), &mut tmp);
            }
        }
    }

    fn write_cast_value(&mut self, key: ffi::ID, value: ffi::VALUE) {
        let new_attr = self.get(key).map(|a| a.with_cast_value(value));
        if let Some(attr) = new_attr {
            self.attributes.insert(key, attr);
        } else {
            missing_attribute(key)
        }
    }

    fn deep_dup(&self) -> Self {
        let attributes = self.attributes
            .iter()
            .map(|(&k, v)| (k, v.deep_dup()))
            .collect();
        Self::new(attributes)
    }

    fn reset(&mut self, key: ffi::ID) {
        if self.has_key(key) {
            self.write_from_database(key, unsafe { ffi::Qnil });
        }
    }

    fn accessed(&self) -> ffi::VALUE {
        let keys = self.attributes
            .values()
            .filter(|a| a.has_been_read())
            .map(Attribute::name);
        to_ruby_array(self.attributes.len(), keys)
    }

    fn map<'a, F: Fn(&'a Attribute) -> Attribute>(&'a self, f: F) -> Self {
        let attributes = self.attributes.iter().map(|(&k, v)| (k, f(v))).collect();
        Self::new(attributes)
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}

fn missing_attribute(key: ffi::ID) -> ! {
    use std::{slice, str};

    unsafe {
        let active_model = ffi::rb_const_get(ffi::rb_cObject, id!("ActiveModel"));
        let missing_attribute = ffi::rb_const_get(active_model, id!("MissingAttributeError"));
        let attr_name = ffi::rb_id2str(key);
        let attr_name_bytes = slice::from_raw_parts(
            ffi::RSTRING_PTR(attr_name) as *const u8,
            ffi::RSTRING_LEN(attr_name) as usize,
        );
        let attr_name = str::from_utf8_unchecked(attr_name_bytes);
        let message = format!("can't write unknown attribute `{}`", attr_name);
        let c_message = CString::new(message).unwrap();
        ffi::rb_raise(missing_attribute, c_message.as_ptr());
    }
}
