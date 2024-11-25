pub mod aws;

pub trait Region {
    fn get_region(&self) -> &str;
}

pub struct NoneRegion;

impl Region for NoneRegion {
    fn get_region(&self) -> &str {
        ""
    }
}