use std::collections::HashMap;

use attribute::Attribute;
use ffi;

mod ruby_glue;

#[derive(Default, Clone)]
pub struct AttributeSet {
    attributes: HashMap<ffi::ID, Attribute>,
}

impl AttributeSet {
    pub fn new(attributes: HashMap<ffi::ID, Attribute>) -> Self {
        Self { attributes }
    }

    fn get(&self, key: ffi::ID) -> Option<&Attribute> {
        self.attributes.get(&key)
    }

    fn write_from_database(&mut self, key: ffi::ID, value: ffi::VALUE) {
        let new_attr = self.get(key).map(|a| a.with_value_from_database(value));
        if let Some(attr) = new_attr {
            self.attributes.insert(key, attr);
        }
    }
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
