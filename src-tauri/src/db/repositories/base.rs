use anyhow::Result;

pub trait Repository<T> {
    fn get(&self, id: &str) -> Result<Option<T>>;
    fn get_all(&self) -> Result<Vec<T>>;
    fn insert(&self, id: &str, item: &T) -> Result<()>;
    fn update(&self, id: &str, item: &T) -> Result<()>;
    fn delete(&self, id: &str) -> Result<()>;
    fn exists(&self, id: &str) -> Result<bool>;
}
