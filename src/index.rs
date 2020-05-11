use crate::page;

use crate::page::{Id, Tag, TagRef};
use anyhow::Result;
use async_trait::async_trait;
use log::info;
use std::collections::{HashMap, HashSet};

/// Indexing wrapper over `page::Store`
///
/// `Index` keeps track of page data neccessary
/// to quickly look them up by a tag query.
#[derive(Default)]
pub struct Index<T> {
    page_ids_by_tag: HashMap<String, HashSet<Id>>,
    tags_by_page_id: HashMap<Id, HashSet<Tag>>,
    title_by_page_id: HashMap<Id, String>,
    store: T,
}

/// Basic page info
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub id: Id,
    pub title: String,
}

/// Results of tag query lookup
#[derive(Default, Debug, Clone)]
pub struct FindResults {
    pub matching_pages: Vec<PageInfo>,
    pub matching_tags: Vec<Tag>,
}

impl FindResults {
    fn empty() -> Self {
        Self::default()
    }
}

/// More compact (post-processed) `FindResults`
pub struct CompactResults {
    // all tags that were not already filtered on
    pub tags: Vec<(Tag, usize)>,
    // all pages that can't be reached by one of the `tags`
    pub pages: Vec<PageInfo>,
}

impl<T> Index<T>
where
    T: page::StoreMut,
{
    pub async fn new(store: T) -> Result<Self> {
        let mut s = Index {
            page_ids_by_tag: Default::default(),
            tags_by_page_id: Default::default(),
            title_by_page_id: Default::default(),
            store,
        };

        s.index_inner().await?;

        Ok(s)
    }

    /// Index the inner `Store`
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

    /// Compact the results to a shorter form
    pub fn compact_results(&self, results: FindResults) -> CompactResults {
        let matching_tags: HashSet<String> = results.matching_tags.iter().cloned().collect();
        let mut unmatched_tags: HashMap<Tag, usize> = Default::default();
        for page_info in &results.matching_pages {
            for page_tag in &self.tags_by_page_id[&page_info.id] {
                if !matching_tags.contains(page_tag.as_str()) {
                    *unmatched_tags.entry(page_tag.to_owned()).or_default() += 1;
                }
            }
        }

        let unmatched_tags_set: HashSet<Tag> = unmatched_tags.keys().cloned().collect();

        let mut pages: Vec<PageInfo> = results
            .matching_pages
            .into_iter()
            .filter(|page_info| {
                unmatched_tags_set
                    .intersection(&self.tags_by_page_id[&page_info.id])
                    .next()
                    .is_none()
            })
            .collect();

        pages.sort_by(|a, b| a.title.cmp(&b.title));

        let mut tags: Vec<_> = unmatched_tags.into_iter().collect();

        tags.sort_by(|a, b| a.1.cmp(&b.1).reverse().then_with(|| a.0.cmp(&b.0)));

        CompactResults { tags, pages }
    }
}

impl<T> Index<T> {
    /// Lookup pages with a list of tags
    pub fn find(&self, tags: &[TagRef]) -> FindResults {
        let mut matching_pages: Vec<PageInfo> = vec![];
        let mut matching_tags: Vec<String> = vec![];
        let mut already_tried_tags = HashSet::new();

        if tags.is_empty() {
            matching_pages = self
                .tags_by_page_id
                .keys()
                .cloned()
                .map(|id| PageInfo {
                    id: id.clone(),
                    title: self.title_by_page_id[&id].clone(),
                })
                .collect();
        }

        for tag in tags {
            if already_tried_tags.contains(tag) {
                continue;
            }
            already_tried_tags.insert(tag);
            if matching_tags.is_empty() {
                if let Some(ids) = &self.page_ids_by_tag.get(*tag) {
                    matching_pages = ids
                        .iter()
                        .map(|id| PageInfo {
                            id: id.to_owned(),
                            title: self.title_by_page_id[id].clone(),
                        })
                        .collect();
                    matching_tags.push(tag.to_string())
                } else {
                    return FindResults::empty();
                }
            } else {
                if let Some(ids) = self.page_ids_by_tag.get(*tag) {
                    let new_matching_pages: Vec<_> = matching_pages
                        .iter()
                        .filter(|info| ids.contains(info.id.as_str()))
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
                .insert(page.id().to_owned());
        }
        self.tags_by_page_id
            .insert(page.id().to_owned(), page.tags.clone());
        self.title_by_page_id
            .insert(page.id().to_owned(), page.title.clone());
    }

    fn clean_data_for_page(&mut self, id: Id) {
        for tag in self
            .tags_by_page_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| HashSet::new())
        {
            self.page_ids_by_tag
                .get_mut(&tag)
                .map(|set| set.remove(&id));
        }
        self.tags_by_page_id.remove(&id);
        self.title_by_page_id.remove(&id);
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

        if let Some(_tags) = self.tags_by_page_id.get(page.id()) {
            self.clean_data_for_page(page.id().to_owned());
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
