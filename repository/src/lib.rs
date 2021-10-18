use std::error::Error;

pub mod github_store;

pub trait Store {
    fn save(&self, s: String) -> Result<(), Box<dyn Error>>;
}
