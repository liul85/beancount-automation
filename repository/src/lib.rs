use anyhow::Result;

pub mod github_store;

pub trait Store {
    fn save(&self, s: String) -> Result<()>;
}
