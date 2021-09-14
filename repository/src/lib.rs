mod github_store;

pub trait BeancountStore {
    fn save(content: String) -> Result<(), String>;
}
