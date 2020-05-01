use crate::page::{self, Id};
use anyhow::{format_err, Result};
use std::collections::HashMap;
// use std::sync::{Arc, Mutex};

pub mod fs;

pub trait Store {
    fn get(&self, id: Id) -> Result<page::Parsed>;
    fn put(&self, page: &page::Parsed) -> Result<()>;
    fn delete(&self, id: Id) -> Result<()>;
    fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>>;
}

pub trait StoreMut {
    fn get(&mut self, id: Id) -> Result<page::Parsed>;
    fn put(&mut self, page: &page::Parsed) -> Result<()>;
    fn delete(&mut self, id: Id) -> Result<()>;
    fn iter<'s>(&'s mut self) -> Result<Box<dyn Iterator<Item = Id> + 's>>;
}

impl<T> StoreMut for T
where
    T: Store,
{
    fn get(&mut self, id: Id) -> Result<page::Parsed> {
        Store::get(self, id)
    }

    fn put(&mut self, page: &page::Parsed) -> Result<()> {
        Store::put(self, page)
    }

    fn delete(&mut self, id: Id) -> Result<()> {
        Store::delete(self, id)
    }

    fn iter<'s>(&'s mut self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        Store::iter(self)
    }
}

// impl Store for Arc<Mutex<InMemoryStore>> {}

#[derive(Debug, Default)]
pub struct InMemoryStore {
    page_by_id: HashMap<Id, page::Parsed>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Default::default()
    }

    /*
    fn inner(&self) -> Result<std::sync::MutexGuard<InMemoryStoreInner>> {
        self.inner
            .lock()
            .map_err(|e| format_err!("Lock failed {}", e))
    }
    */
}

impl StoreMut for InMemoryStore {
    fn get(&mut self, id: Id) -> Result<page::Parsed> {
        Ok(self
            .page_by_id
            .get(&id)
            .cloned()
            .ok_or_else(|| format_err!("Not found"))?)
    }

    fn put(&mut self, page: &page::Parsed) -> Result<()> {
        *self
            .page_by_id
            .get_mut(&page.headers.id)
            .ok_or_else(|| format_err!("Not found"))? = page.clone();

        Ok(())
    }

    fn delete(&mut self, id: Id) -> Result<()> {
        self.page_by_id
            .remove(&id)
            .ok_or_else(|| format_err!("Not found"))?;
        Ok(())
    }
    fn iter<'s>(&'s mut self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        Ok(Box::new(self.page_by_id.keys().cloned()))
    }
}
