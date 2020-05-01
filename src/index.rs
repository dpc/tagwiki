use crate::page;

use crate::page::{Id, Tag};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct Index<T> {
    // tag -> page_ids
    page_ids_by_tag: HashMap<String, HashSet<Id>>,
    tags_by_page_id: HashMap<Id, Vec<Tag>>,
    inner: T,
}

#[derive(Default)]
struct FindResults {
    matching_pages: Vec<Id>,
    matching_tags: Vec<page::Tag>,
}

impl FindResults {
    fn empty() -> Self {
        Self::default()
    }
}

impl<T> Index<T> {
    fn find(&self, tags: &[&Tag]) -> FindResults {
        let mut matching_pages = vec![];
        let mut matching_tags = vec![];
        for tag in tags {
            if matching_tags.is_empty() {
                if let Some(ids) = self.page_ids_by_tag.get(tag.as_str()) {
                    matching_pages = ids.iter().map(|id| id.to_owned()).collect();
                } else {
                    return FindResults::empty();
                }
            } else {
                if let Some(ids) = self.page_ids_by_tag.get(tag.as_str()) {
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

impl<T> page::StoreMut for Index<T>
where
    T: page::StoreMut,
{
    fn get(&mut self, id: Id) -> Result<page::Parsed> {
        self.inner.get(id)
    }
    fn put(&mut self, page: &page::Parsed) -> Result<()> {
        self.inner.put( page)?;

        if let Some(_tags) = self.tags_by_page_id.get(&page.headers.id) {
            self.clean_data_for_page(page.headers.id.clone());
        }

        for tag in &page.tags {
            self.page_ids_by_tag
                .get_mut(tag)
                .map(|set| set.insert(page.headers.id.clone()));
            self.tags_by_page_id
                .insert(page.headers.id.clone(), page.tags.clone());
        }
        Ok(())
    }
    fn delete(&mut self, id: Id) -> Result<()> {
        self.inner.delete(id.clone())?;
        self.clean_data_for_page(id);

        Ok(())
    }

    fn iter<'s>(&'s mut self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        self.inner.iter()
    }
}
