use anyhow::Result;
use beancount_core::parser::Transaction;

pub mod github_store;

pub trait Store {
    fn save(&self, transaction: Transaction) -> Result<String>;
}
