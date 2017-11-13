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
}

pub unsafe fn init() {
    self::ruby_glue::init();
}
