pub mod aws;

pub trait Region {
    fn get_region(&self) -> &str;
}