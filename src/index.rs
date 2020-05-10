use crate::page;

use crate::page::{Id, Tag, TagRef};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Index<T> {
    // tag -> page_ids
    page_ids_by_tag: HashMap<String, HashSet<Id>>,
    tags_by_page_id: HashMap<Id, Vec<Tag>>,
    store: T,
}

#[derive(Default, Debug, Clone)]
pub struct FindResults {
    pub matching_pages: Vec<Id>,
    pub matching_tags: Vec<page::Tag>,
}

impl FindResults {
    fn empty() -> Self {
        Self::default()
    }
}

impl<T> Index<T>
where
    T: page::StoreMut,
{
    pub async fn new(store: T) -> Result<Self> {
        let mut s = Index {
            page_ids_by_tag: Default::default(),
            tags_by_page_id: Default::default(),
            store,
        };

        s.index_inner().await?;

        Ok(s)
    }

    async fn index_inner(&mut self) -> Result<()> {
        let mut count = 0;
        let ids = self.store.iter().await?.collect::<Vec<page::Id>>();
        for id in ids {
            count += 1;
            let page = self.store.get(id).await?;
            self.add_data_for_page(&page);
        }
        info!("Indexed {} pages", count);
        Ok(())
    }
}

impl<T> Index<T> {
    pub fn find(&self, tags: &[TagRef]) -> FindResults {
        let mut matching_pages: Vec<String> = vec![];
        let mut matching_tags: Vec<String> = vec![];
        let mut already_tried_tags = HashSet::new();
        for tag in tags {
            if already_tried_tags.contains(tag) {
                continue;
            }
            already_tried_tags.insert(tag);
            if matching_tags.is_empty() {
                if let Some(ids) = &self.page_ids_by_tag.get(*tag) {
                    matching_pages = ids.iter().map(|id| id.to_owned()).collect();
                    matching_tags.push(tag.to_string())
                } else {
                    return FindResults::empty();
                }
            } else {
                if let Some(ids) = self.page_ids_by_tag.get(*tag) {
                    let new_matching_pages: Vec<_> = matching_pages
                        .iter()
                        .filter(|id| ids.contains(id.as_str()))
                        .map(|id| id.to_owned())
                        .collect();
                    if new_matching_pages.is_empty() {
                        return FindResults {
                            matching_pages,
                            matching_tags,
                        };
                    }

                    matching_pages = new_matching_pages;
                    matching_tags.push(tag.to_string());
                } else {
                    return FindResults {
                        matching_pages,
                        matching_tags,
                    };
                }
            }
        }
        FindResults {
            matching_pages,
            matching_tags,
        }
    }

    fn add_data_for_page(&mut self, page: &page::Parsed) {
        for tag in &page.tags {
            self.page_ids_by_tag
                .entry(tag.clone())
                .or_default()
                .insert(page.headers.id.clone());
            self.tags_by_page_id
                .insert(page.headers.id.clone(), page.tags.clone());
        }
    }

    fn clean_data_for_page(&mut self, id: Id) {
        for tag in self
            .tags_by_page_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| vec![])
        {
            self.page_ids_by_tag
                .get_mut(&tag)
                .map(|set| set.remove(&id));
        }
        self.tags_by_page_id.remove(&id);
    }
}

#[async_trait]
impl<T> page::StoreMut for Index<T>
where
    T: page::StoreMut + Send + Sync,
{
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        self.store.get(id).await
    }

    async fn put(&mut self, page: &page::Parsed) -> Result<()> {
        self.store.put(page).await?;

        if let Some(_tags) = self.tags_by_page_id.get(&page.headers.id) {
            self.clean_data_for_page(page.headers.id.clone());
        }

        self.add_data_for_page(page);
        Ok(())
    }

    async fn delete(&mut self, id: Id) -> Result<()> {
        self.store.delete(id.clone()).await?;
        self.clean_data_for_page(id);

        Ok(())
    }

    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        self.store.iter().await
    }
}
