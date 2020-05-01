mod store;

use lazy_static::lazy_static;
pub use store::{InMemoryStore, Store, StoreMut};

use anyhow::Result;
use digest::Digest;

pub type Id = String;
pub type Tag = String;

const TAGWIKI_PAGE_ID_KEY: &str = "tagwiki-page-id";

#[derive(Debug, Default, Clone)]
pub struct Source(String);

#[derive(Debug, Default, Clone)]
pub struct Parsed {
    pub source: Source,
    pub html: String,
    pub headers: Headers,
    pub tags: Vec<Tag>,
    pub title: String,
}

fn split_headers_and_body(source: &Source) -> (&str, &str) {
    lazy_static! {
        static ref RE: regex::Regex =
            regex::RegexBuilder::new(r"\A[[:space:]]*<!--+(.*)--+>(.*)\z")
                .multi_line(true)
                .dot_matches_new_line(true)
                .build()
                .unwrap();
    }

    if let Some(cap) = RE.captures_iter(&source.0).next() {
        (
            cap.get(1).expect("be there").as_str(),
            cap.get(2).expect("be there").as_str(),
        )
    } else {
        ("", &source.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Headers {
    pub id: String,
    pub all: String,
}

impl Headers {
    fn parse(headers_str: &str, source: &Source) -> Headers {
        let mut id = None;

        for line in headers_str.lines() {
            match line.split(":").collect::<Vec<_>>().as_slice() {
                [key, value] => {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        TAGWIKI_PAGE_ID_KEY => {
                            id = Some(value.to_owned());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        match id {
            Some(id) => Self {
                id,
                all: headers_str.to_owned(),
            },
            None => {
                let mut hasher = blake2::Blake2b::new();
                hasher.input(&source.0);
                let res = hasher.result();
                let id = hex::encode(&res.as_slice()[0..16]);

                let mut all = String::new();
                all.push_str(TAGWIKI_PAGE_ID_KEY);
                all.push_str(": ");
                all.push_str(&id);
                all.push_str("\n");
                all.push_str(headers_str);

                Self { id, all }
            }
        }
    }
}

impl Parsed {
    fn from_markdown(source: Source) -> Parsed {
        let (headers, body) = split_headers_and_body(&source);
        let headers = Headers::parse(headers, &source);

        let parser = pulldown_cmark::Parser::new(body);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        Parsed {
            headers,
            html: html_output,
            source,
            tags: vec!["TODO".into()],
            title: "TODO".into(),
        }
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
    let page = Parsed::from_markdown(Source(
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
    assert_eq!(page.headers.id, "xyz");
    Ok(())
}

fn add_to_store(_store: &impl Store, source: Source) -> Result<()> {
    let _page = Parsed::from_markdown(source);
    Ok(())
}
