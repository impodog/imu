mod prelude;
pub mod rules;

pub type Name = String;
pub trait ToName {
    fn to_name(&self) -> Name;
}

impl ToName for str {
    fn to_name(&self) -> Name {
        self.to_owned()
    }
}
