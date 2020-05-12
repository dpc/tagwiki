use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use horrorshow::{box_html, owned_html};

use crate::index;
use crate::page::{self, Tag};

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
           : body
        }
    }
}

pub fn page(page: Option<&page::Parsed>, edit: bool) -> Box<dyn RenderBox> {
    if edit {
        Box::new(page_editing_view(page)) as Box<dyn RenderBox>
    } else {
        Box::new(page_view(page.expect("always some"))) as Box<dyn RenderBox>
    }
}

pub fn page_editing_view(page: Option<&page::Parsed>) -> impl RenderOnce {
    if let Some(page) = page.as_ref() {
        let body = page.source_body.clone();
        let id = page.id().to_owned();
        (box_html! {
            form(action=".", method="post", class="pure-form") {
                a(href=format!("?id={}", id),class="pure-button"){ : "Cancel" }
                : " ";
                input(type="submit", value="Save", class="pure-button pure-button-primary");
                input(type="hidden", name="id", value=id);
                textarea(name="body") {
                    : body
                }
            }
        }) as Box<dyn RenderBox>
    } else {
        box_html! {
            form(action=".", method="post", class="pure-form") {
                a(href="javascript:history.back()",class="pure-button"){ : "Cancel" }
                : " ";
                input(type="submit", value="Save", class="pure-button pure-button-primary");
                input(type="hidden", name="_method", value="put");
                textarea(name="body");
            }
        }
    }
}

pub fn page_view(page: &page::Parsed) -> impl RenderOnce {
    let page_html = page.html.clone();
    let id = page.id().to_owned();
    let id_copy = id.clone();
    owned_html! {
        div(class="pure-menu pure-menu-horizontal") {
            form(action="..", method="get", class="pure-menu-item pure-form") {
                button(type="submit", class="pure-button"){
                    : "Up"
                }
            }
            : " ";
            form(action="/", method="get", class="pure-menu-item pure-form") {
                input(type="hidden", name="edit", value="true");
                button(type="submit", class="pure-button button-green"){
                    : "New"
                }
            }
            : " ";
            form(action=".", method="get", class="pure-menu-item pure-form") {
                input(type="hidden", name="edit", value="true");
                input(type="hidden", name="id", value=id);
                button(type="submit", class="pure-button pure-button-primary"){
                    : "Edit"
                }
            }
            : " ";
            form(action=".", method="post", class="pure-menu-item pure-form") {
                input(type="hidden", name="edit", value="true");
                input(type="hidden", name="id", value=id_copy);
                input(type="hidden", name="_method", value="delete");
                button(type="submit", class="pure-button button-warning",onclick="return confirm('Are you sure?');"){
                    : "Delete"
                }
            }
        }
        : Raw(page_html)
    }
}

pub fn post_list(
    unmatched_tags: impl Iterator<Item = (Tag, usize)>,
    posts: impl Iterator<Item = index::PageInfo> + 'static,
) -> impl RenderOnce {
    owned_html! {
        div(class="pure-menu pure-menu-horizontal") {
            form(action="..", method="get", class="pure-menu-item pure-form") {
                button(type="submit", class="pure-button"){
                    : "Up"
                }
            }
            : " ";
            form(action="/", method="get", class="pure-menu-item pure-form") {
                input(type="hidden", name="edit", value="true");
                button(type="submit", class="pure-button button-green"){
                    : "New"
                }
            }
        }
        ul {
            @ for tag in unmatched_tags {
                li {
                    a(href=format!("./{}/", tag.0)) : format!("{} ({})", tag.0, tag.1)
                }
            }
            @ for post in posts {
                li {
                    a(href=format!("./?id={}", post.id)) : post.title
                }
            }
        }
    }
}
