use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use horrorshow::{box_html, owned_html};

use crate::index;
use crate::page::{Parsed, Tag};

#[derive(Clone, Debug)]
pub struct PageState {
    // currently rendered path
    pub path: String,
    pub edit: bool,
    pub page: Option<Parsed>,
    pub subtags: Vec<(String, usize)>,
}

pub fn html_page(body: impl RenderOnce) -> impl RenderOnce {
    owned_html! {
        : doctype::HTML;
        head {
            link(rel="stylesheet",href="https://unpkg.com/purecss@2.0.1/build/pure-min.css",crossorigin="anonymous");
            link(rel="stylesheet",href="https://unpkg.com/purecss@2.0.1/build/grids-responsive-min.css");
            meta(name="viewport",content="width=device-width, initial-scale=1");
            link(rel="stylesheet", media="all", href="/_style.css");
        }
        body {
           : body;
           script(src="/_script.js");
        }
    }
}

pub fn page(page_state: PageState) -> Box<dyn RenderBox> {
    if page_state.edit.clone() {
        Box::new(page_editing_view(page_state.clone())) as Box<dyn RenderBox>
    } else {
        let page_state_clone = page_state.clone();
        let sub_pages = owned_html! {
            @ if !page_state_clone.subtags.is_empty() {
                h1 { : "Subpages" }
                ul {
                    @ for tag in &page_state_clone.subtags {
                        li {
                            a(href=format!("./{}/", tag.0)) : format!("{} ({})", tag.0, tag.1)
                        }
                    }
                }
            }
        };
        Box::new(page_view(page_state.clone(), sub_pages)) as Box<dyn RenderBox>
    }
}

pub fn page_editing_view(page_state: PageState) -> impl RenderOnce {
    if let Some(page) = page_state.page.as_ref() {
        let body = page.source_body.clone();
        menu(
            page_state.clone(),
            Some(
                (box_html! {
                    textarea(name="body", id="source-editor", class="append", autofocus) {
                        : body
                    }
                }) as Box<dyn RenderBox>,
            ),
        )
    } else {
        let starting_tags = page_state
            .path
            .split("/")
            .filter(|t| !t.trim().is_empty())
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join(" ");
        let starting_text = "\n\n\n".to_string() + &starting_tags;
        menu(
            page_state.clone(),
            Some(
                (box_html! {
                    textarea(name="body", id="source-editor", class="prepend", autofocus) {
                        : starting_text
                    }
                }) as Box<dyn RenderBox>,
            ),
        )
    }
}

pub fn menu(page_state: PageState, subform: Option<Box<dyn RenderBox>>) -> impl RenderOnce {
    let id = page_state.page.map(|p| p.id().to_owned());
    let edit = page_state.edit;
    let path_tags: String = page_state
        .path
        .split("/")
        .filter(|f| !f.trim().is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    // # The sucky menu mega-form
    // I really want one top-bar with all the buttons, and because I want
    // everything to work even without JS enabled, all stuff here is quirky
    // * GET queries are just links to avoid conflicting with other inputs
    // * other buttons use `_method=METHOD` and `formaction` and `formmethod` + server-side redirect
    // * query text uses a server-side redirect on `q`
    // If more stuff is cramed in here, this will all eventually fall appart. :)
    owned_html! {
        form(class="pure-form") {
            div(class="pure-menu pure-menu-horizontal") {
                @ if let Some(id) = id.as_deref() {
                    input(type="hidden", name="id", value=id);
                }

                @ if edit {
                    @ if let Some(id) = id.as_deref() {
                        a(href=format!("?id={}", id), class="pure-button", id="cancel-button"){ : "Cancel" }
                        : " ";
                    } else {
                        a(href="javascript:history.back()", class="pure-button", id="cancel-button"){ : "Cancel" }
                        : " ";
                    }
                } else {
                    a(href="..",class="pure-button", id="up-button") { : "Up" }
                    : " ";
                }
                @ if edit {
                    @ if let Some(_id) = id.as_deref() {
                        button(type="submit", id="save-button", class="pure-button pure-button-primary", formaction=".", formmethod="post"){
                            : Raw("<u>S</u>ave")
                        }
                    } else {
                        button(type="submit", id="save-button", class="pure-button pure-button-primary", formaction=".", formmethod="post", name="_method", value="put"){
                            : Raw("<u>S</u>ave")
                        }
                    }
                    : " ";
                } else {
                    a(href="?edit=true", id="new-button", class="pure-button button-green"){ : Raw("<u>N</u>ew") }
                    : " ";
                }
                @ if !edit && id.is_some() {
                    a(type="submit", href=format!("?id={}&edit=true", id.as_ref().unwrap()), id="edit-button", class="pure-button pure-button-primary"){
                        : Raw("<u>E</u>dit")
                    }
                    : " ";
                    button(type="submit", id="delete-button", class="pure-button button-warning", formaction=".", formmethod="post", name="_method", value="delete", onclick="return confirm('Are you sure?');"){
                        : Raw("<u>D</u>elete")
                    }
                }
                : " ";
                button(type="submit", id="query-button", class="pure-button float-right", formaction="/_query", formmethod="get") {
                    : "Search"
                }
                input(type="text", class="float-right", id="query-text", name="q", placeholder="tag1 tag2...", value=path_tags);
            }
            : subform
        }
    }
}

pub fn page_view(page_state: PageState, sub_pages: impl RenderOnce) -> impl RenderOnce {
    let menu = menu(page_state.clone(), None);
    let page = page_state.page.expect("always some");
    let page_html = page.html.clone();
    owned_html! {
        : menu;
        article(id="page-content") {
            : Raw(page_html);
            : sub_pages;
        }
    }
}

pub fn post_list(
    page_state: PageState,
    unmatched_tags: impl Iterator<Item = (Tag, usize)>,
    posts: impl Iterator<Item = index::PageInfo> + 'static,
) -> impl RenderOnce {
    let menu = menu(page_state.clone(), None);
    owned_html! {
        : menu;
        div(id="page-content") {
            h1 { : "Subpages" }
            ul(id="index") {
                @ for post in posts {
                    li {
                        a(href=format!("./?id={}", post.id)) : post.title
                    }
                }
                @ for tag in unmatched_tags {
                    li {
                        a(href=format!("./{}/", tag.0)) : format!("{} ({})", tag.0, tag.1)
                    }
                }
            }
        }
    }
}
