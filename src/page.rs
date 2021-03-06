pub mod store;

use crate::util;
#[allow(unused)]
use anyhow::Result;
use chrono::prelude::*;
use lazy_static::lazy_static;
use std::collections::HashSet;
pub use store::{InMemoryStore, Store, StoreMut};

use digest::Digest;

pub type Id = String;
pub type Tag = String;
pub type TagRef<'a> = &'a str;
pub type IdRef<'a> = &'a str;

const TAGWIKI_PAGE_ID_KEY: &str = "tagwiki-page-id";
const TAGWIKI_CREATION_TIME_KEY: &str = "tagwiki-creation-time";
const TAGWIKI_MODIFICATION_TIME_KEY: &str = "tagwiki-modification-time";

#[derive(Debug, Default, Clone)]
pub struct Source(String);

#[derive(Debug, Clone)]
pub struct Parsed {
    pub source: Source,
    pub source_body: String,
    pub html: String,
    pub headers: Headers,
    pub tags: HashSet<Tag>,
    pub title: String,
}

fn split_headers_and_body(source: &Source) -> (&str, &str) {
    lazy_static! {
        static ref RE: regex::Regex =
            regex::RegexBuilder::new(r"\A[[:space:]]*<!--+(.*?)--+>(.*)\z")
                .multi_line(true)
                .dot_matches_new_line(true)
                .build()
                .unwrap();
    }

    if let Some(cap) = RE.captures_iter(&source.0).next() {
        (
            // important: trimming headers, prevent them from accumulating newlines in the output
            // during rewrites
            cap.get(1).expect("be there").as_str().trim(),
            cap.get(2).expect("be there").as_str(),
        )
    } else {
        ("", &source.0)
    }
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub id: String,
    pub creation_time: chrono::DateTime<FixedOffset>,
    pub modification_time: chrono::DateTime<FixedOffset>,
    pub other: String,
}

impl Default for Headers {
    fn default() -> Self {
        Self {
            id: util::random_string(16),
            creation_time: util::now(),
            modification_time: util::now(),
            other: "".into(),
        }
    }
}
impl Headers {
    fn parse(headers_str: &str, source: &Source) -> Headers {
        let mut id = None;
        let mut creation = None;
        let mut modification = None;
        let mut other = String::new();

        for line in headers_str.lines() {
            match line.splitn(2, ":").collect::<Vec<_>>().as_slice() {
                [key, value] => {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        TAGWIKI_PAGE_ID_KEY => {
                            id = Some(value.to_owned());
                        }
                        TAGWIKI_CREATION_TIME_KEY => {
                            let time = chrono::DateTime::<FixedOffset>::parse_from_rfc3339(value);
                            creation = time.ok();
                        }
                        TAGWIKI_MODIFICATION_TIME_KEY => {
                            let time = chrono::DateTime::<FixedOffset>::parse_from_rfc3339(value);
                            modification = time.ok();
                        }
                        _ => {
                            other.push_str(line);
                            other.push_str("\n")
                        }
                    }
                }
                _ => {
                    other.push_str(line);
                    other.push_str("\n")
                }
            }
        }

        let id = id.unwrap_or_else(|| {
            let mut hasher = blake2::Blake2b::new();
            hasher.update(&source.0);
            let res = hasher.finalize();
            hex::encode(&res.as_slice()[0..16])
        });

        let creation: DateTime<chrono::offset::FixedOffset> =
            creation.unwrap_or_else(|| util::now());

        let modification = modification.unwrap_or_else(|| util::now());

        Self {
            other,
            id,
            creation_time: creation,
            modification_time: modification,
        }
    }

    fn to_markdown_string(&self) -> String {
        "<!---\n".to_string()
            + &format!("{}: {}\n", TAGWIKI_PAGE_ID_KEY, self.id)
            + &format!(
                "{}: {}\n",
                TAGWIKI_CREATION_TIME_KEY,
                self.creation_time.to_rfc3339()
            )
            + &format!(
                "{}: {}\n",
                TAGWIKI_MODIFICATION_TIME_KEY,
                self.modification_time.to_rfc3339()
            )
            + &self.other
            + "-->\n"
    }
}

fn parse_tags(body: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: regex::Regex =
            regex::Regex::new(r"#([a-zA-Z0-9_\-]+)").expect("correct regex");
    }

    RE.captures_iter(&body)
        .map(|m| m.get(1).expect("a value").as_str().to_lowercase())
        .collect()
}

fn parse_title(body: &str) -> String {
    lazy_static! {
        static ref RE: regex::Regex =
            regex::Regex::new(r"#+[[:space:]]+(.*)").expect("correct regex");
    }

    let title = RE
        .captures_iter(&body)
        .map(|m| m.get(1).expect("a value").as_str().trim().to_string())
        .next()
        .unwrap_or_else(|| "".to_string());
    let title = if title == "" {
        lazy_static! {
            static ref RE: regex::Regex =
                regex::Regex::new(r"[[:space:]]*(.*?)[\.\n]").expect("correct regex");
        }

        RE.captures_iter(&body)
            .map(|m| m.get(1).expect("a value").as_str().trim().to_string())
            .next()
            .unwrap_or_else(|| "".to_string())
    } else {
        title
    };

    if title == "" {
        "Untitled".to_string()
    } else {
        title
    }
}

impl Parsed {
    pub fn id(&self) -> IdRef {
        self.headers.id.as_str()
    }

    pub fn new(body: &str) -> Parsed {
        let headers = Headers {
            id: crate::util::random_string(16),
            ..Headers::default()
        };
        Self::from_headers_and_body(headers, body.to_owned())
    }

    pub fn from_full_source(source: Source) -> Parsed {
        let (headers, body) = split_headers_and_body(&source);
        let headers = Headers::parse(headers, &source);

        Self::from_headers_and_body(headers, body.to_owned())
    }

    fn from_headers_and_body(headers: Headers, body: String) -> Parsed {
        let source = headers.to_markdown_string() + &body;
        let parser = pulldown_cmark::Parser::new(&body);
        let title = parse_title(&body);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        let tags = parse_tags(&body);

        Parsed {
            headers,
            html: html_output,
            source_body: body,
            source: Source(source),
            tags: tags.into_iter().collect(),
            title,
        }
    }

    pub fn update_modification_time(&mut self) {
        self.headers.modification_time = util::now();
    }

    pub fn with_new_source_body(&self, new_body_source: &str) -> Self {
        Self::from_headers_and_body(self.headers.clone(), new_body_source.to_owned())
    }
}

#[test]
fn split_headers_and_body_test() -> Result<()> {
    let s = Source(
        r#"


<!------- a: b
c: d -->banana"#
            .into(),
    );
    let (headers, body) = split_headers_and_body(&s);

    assert_eq!(
        headers,
        r#" a: b
c: d "#
    );
    assert_eq!(body, "banana");

    Ok(())
}

#[test]
fn parse_markdown_metadata_test() -> Result<()> {
    let page = Parsed::from_full_source(Source(
        r#"

<!---

a: b
tagwiki-page-id: xyz

foo-bar: bar
-->
bar
<!---
tagwiki-id: 123
-->
        "#
        .to_owned(),
    ));

    println!("{:#?}", page);
    assert_eq!(page.id(), "xyz");
    Ok(())
}
