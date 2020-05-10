use crate::page::{self, Id};
use anyhow::{format_err, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync;

pub mod fs;
pub use fs::FsStore;

#[async_trait]
pub trait Store {
    async fn get(&self, id: Id) -> Result<page::Parsed>;
    async fn put(&self, page: &page::Parsed) -> Result<()>;
    async fn delete(&self, id: Id) -> Result<()>;
    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>>;
}

#[async_trait]
pub trait StoreMut {
    async fn get(&self, id: Id) -> Result<page::Parsed>;
    async fn put(&mut self, page: &page::Parsed) -> Result<()>;
    async fn delete(&mut self, id: Id) -> Result<()>;
    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>>;
}

#[async_trait]
impl<T> StoreMut for T
where
    T: Store + Send + Sync,
{
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        Store::get(self, id).await
    }

    async fn put(&mut self, page: &page::Parsed) -> Result<()> {
        Store::put(self, page).await
    }

    async fn delete(&mut self, id: Id) -> Result<()> {
        Store::delete(self, id).await
    }

    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        Store::iter(self).await
    }
}

#[async_trait]
impl StoreMut for Box<dyn StoreMut + Send + Sync> {
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        (**self).get(id).await
    }

    async fn put(&mut self, page: &page::Parsed) -> Result<()> {
        (**self).put(page).await
    }

    async fn delete(&mut self, id: Id) -> Result<()> {
        (**self).delete(id).await
    }

    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        (**self).iter().await
    }
} /*
  impl<T> Store for sync::Arc<sync::Mutex<T>>
  where
      T: StoreMut,
  {
      fn get(&self, id: Id) -> Result<page::Parsed> {
          self.lock().expect("locking").get(id)
      }

      fn put(&self, page: &page::Parsed) -> Result<()> {
          self.lock().expect("locking").put(page)
      }

      fn delete(&self, id: Id) -> Result<()> {
          self.lock().expect("locking").delete(id)
      }

      fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
          Ok(Box::new(
              self.lock()
                  .expect("locking")
                  .iter()?
                  .collect::<Vec<_>>()
                  .into_iter(),
          ))
      }
  }
  */

#[async_trait]
impl<T> Store for sync::Arc<tokio::sync::RwLock<T>>
where
    T: StoreMut + Sync + Send,
{
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        self.read().await.get(id).await
    }

    async fn put(&self, page: &page::Parsed) -> Result<()> {
        self.write().await.put(page).await
    }

    async fn delete(&self, id: Id) -> Result<()> {
        self.write().await.delete(id).await
    }

    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        // TODO: fix that `collect`
        Ok(Box::new(
            self.write()
                .await
                .iter()
                .await?
                .collect::<Vec<_>>()
                .into_iter(),
        ))
    }
} // impl Store for Arc<Mutex<InMemoryStore>> {}

#[derive(Debug, Default)]
pub struct InMemoryStore {
    page_by_id: HashMap<Id, page::Parsed>,
}

impl InMemoryStore {
    #[allow(unused)]
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl StoreMut for InMemoryStore {
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        Ok(self
            .page_by_id
            .get(&id)
            .cloned()
            .ok_or_else(|| format_err!("Not found"))?)
    }

    async fn put(&mut self, page: &page::Parsed) -> Result<()> {
        *self
            .page_by_id
            .get_mut(&page.headers.id)
            .ok_or_else(|| format_err!("Not found"))? = page.clone();

        Ok(())
    }

    async fn delete(&mut self, id: Id) -> Result<()> {
        self.page_by_id
            .remove(&id)
            .ok_or_else(|| format_err!("Not found"))?;
        Ok(())
    }
    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        Ok(Box::new(self.page_by_id.keys().cloned()))
    }
}
