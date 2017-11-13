use ordermap::OrderMap;

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

    fn get(&self, key: ffi::ID) -> Option<&Attribute> {
        self.attributes.get(&key)
    }

    fn set(&mut self, key: ffi::ID, attr: Attribute) {
        self.attributes.insert(key, attr);
    }

    fn values_before_type_cast(&self) -> ffi::VALUE {
        let result = unsafe { ffi::rb_hash_new() };
        for (&key, value) in &self.attributes {
            let key = unsafe { ffi::rb_id2sym(key) };
            unsafe { ffi::rb_hash_aset(result, key, value.value_before_type_cast()) };
        }
        result
    }

    fn to_hash(&mut self) -> ffi::VALUE {
        let result = unsafe { ffi::rb_hash_new() };
        let attributes = self.attributes
            .iter_mut()
            .filter(|&(_, ref attr)| attr.is_initialized());
        for (&key, value) in attributes {
            let key = unsafe { ffi::rb_id2sym(key) };
            unsafe { ffi::rb_hash_aset(result, key, value.value()) };
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

    fn fetch_value(&mut self, key: ffi::ID) -> Option<ffi::VALUE> {
        self.attributes.get_mut(&key).map(Attribute::value)
    }

    fn write_from_database(&mut self, key: ffi::ID, value: ffi::VALUE) {
        let new_attr = self.get(key).map(|a| a.with_value_from_database(value));
        if let Some(attr) = new_attr {
            self.attributes.insert(key, attr);
        }
    }

    fn write_from_user(&mut self, key: ffi::ID, value: ffi::VALUE) {
        let new_attr = self.get(key).map(|a| a.with_value_from_user(value));
        if let Some(attr) = new_attr {
            self.attributes.insert(key, attr);
        }
    }

    fn deep_dup(&self) -> Self {
        let attributes = self.attributes
            .iter()
            .map(|(&k, v)| (k, v.deep_dup()))
            .collect();
        Self::new(attributes)
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
