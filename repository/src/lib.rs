use anyhow::Result;
use parser::Transaction;

pub mod github_store;

pub trait Store {
    fn save(&self, transaction: &Transaction) -> Result<()>;
}
