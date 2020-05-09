use crate::page::{self, Id};
use anyhow::{format_err, Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct FsStore {
    root_path: PathBuf,
    id_to_path: HashMap<Id, PathBuf>,
    path_to_page: HashMap<PathBuf, page::Parsed>,
}

impl FsStore {
    pub fn new(root_path: PathBuf) -> Result<Self> {
        let mut s = Self {
            root_path,
            ..Self::default()
        };
        for entry in walkdir::WalkDir::new(&s.root_path) {
            match Self::try_reading_page_from_entry_res(entry) {
                Ok(Some((page, path))) => {
                    s.id_to_path.insert(page.headers.id.clone(), path.clone());
                    s.path_to_page.insert(path, page);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Error reading pages: {}", e);
                }
            }
        }
        Ok(s)
    }

    fn title_to_new_rel_path(&self, title: &str) -> PathBuf {
        let mut last_char_was_alphanum = false;
        let mut path_str = String::new();
        for ch in title.chars() {
            let is_alphanum = ch.is_alphanumeric();

            match (is_alphanum, last_char_was_alphanum) {
                (true, _) => {
                    path_str.push(ch);
                }
                (false, true) => {
                    path_str.push('-');
                }
                (false, false) => {}
            }

            last_char_was_alphanum = is_alphanum;
        }

        let initial_title = path_str.clone();
        let mut path = PathBuf::from(&initial_title);
        let mut i = 1;
        while let Some(_) = self.path_to_page.get(&path) {
            path = PathBuf::from(format!("{}-{}", &initial_title, i));
            i += 1;
        }
        path
    }

    fn try_reading_page_from_entry_res(
        entry: walkdir::Result<walkdir::DirEntry>,
    ) -> Result<Option<(page::Parsed, PathBuf)>> {
        let entry = entry?;
        Self::try_reading_page_from_entry(&entry)
            .with_context(|| format!("While reading path: {}", entry.path().display()))
    }

    fn try_reading_page_from_entry(
        entry: &walkdir::DirEntry,
    ) -> Result<Option<(page::Parsed, PathBuf)>> {
        if !entry.file_type().is_file() {
            return Ok(None);
        }

        if entry.path().extension() != Some(&OsString::from("md")) {
            return Ok(None);
        }

        let file = std::fs::File::open(PathBuf::from(entry.path()))?;
        let mut reader = std::io::BufReader::new(file);
        let mut source = page::Source::default();
        reader.read_to_string(&mut source.0)?;

        Ok(Some((
            page::Parsed::from_markdown(source),
            entry.path().to_owned(),
        )))
    }

    fn write_page_to_file(&self, _rel_path: &Path, _page: &page::Parsed) -> Result<()> {
        todo!();
    }
}

#[async_trait]
impl page::StoreMut for FsStore {
    async fn get(&self, id: Id) -> Result<page::Parsed> {
        self.id_to_path
            .get(&id)
            .and_then(|path| self.path_to_page.get(path).cloned())
            .ok_or_else(|| format_err!("Not found"))
    }

    async fn put(&mut self, page: &page::Parsed) -> Result<()> {
        let path = if let Some(path) = self.id_to_path.get(&page.headers.id) {
            path.clone()
        } else {
            self.title_to_new_rel_path(&page.title)
        };

        self.write_page_to_file(&path, &page)?;
        self.id_to_path
            .insert(page.headers.id.clone(), path.clone());
        self.path_to_page.insert(path, page.clone());
        Ok(())
    }

    async fn delete(&mut self, id: Id) -> Result<()> {
        let path = self
            .id_to_path
            .get(&id)
            .cloned()
            .ok_or_else(|| format_err!("Not found"))?;
        self.path_to_page.remove(&path);
        self.id_to_path.remove(&id);
        std::fs::remove_file(self.root_path.join(path))?;
        Ok(())
    }

    async fn iter<'s>(&'s self) -> Result<Box<dyn Iterator<Item = Id> + 's>> {
        Ok(Box::new(self.id_to_path.keys().cloned()))
    }
}
