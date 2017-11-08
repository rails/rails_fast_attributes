mod ruby_glue;

#[derive(Default)]
pub struct Attribute;

pub unsafe fn init() {
    self::ruby_glue::init();
}
