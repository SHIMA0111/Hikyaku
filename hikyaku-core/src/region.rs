pub trait BaseRegion {
    fn region_name(&self) -> &str;
}

pub struct NoneRegion;

impl BaseRegion for NoneRegion {
    fn region_name(&self) -> &str {
        ""
    }
}