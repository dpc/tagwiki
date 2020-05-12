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
           : body
        }
    }
}

pub fn page(page_state: PageState) -> Box<dyn RenderBox> {
    if page_state.edit {
        Box::new(page_editing_view(page_state)) as Box<dyn RenderBox>
    } else {
        Box::new(page_view(page_state)) as Box<dyn RenderBox>
    }
}

pub fn page_editing_view(page_state: PageState) -> impl RenderOnce {
    if let Some(page) = page_state.page.as_ref() {
        let body = page.source_body.clone();
        menu(
            page_state.clone(),
            Some(
                (box_html! {
                    textarea(name="body") {
                        : body
                    }
                }) as Box<dyn RenderBox>,
            ),
        )
    } else {
        menu(
            page_state.clone(),
            Some(
                (box_html! {
                    textarea(name="body");
                }) as Box<dyn RenderBox>,
            ),
        )
    }
}

pub fn menu(page_state: PageState, subform: Option<Box<dyn RenderBox>>) -> impl RenderOnce {
    let id = page_state.page.map(|p| p.id().to_owned());
    let edit = page_state.edit;

    owned_html! {
        form {
            div(class="pure-menu pure-menu-horizontal") {
                @ if let Some(id) = id.as_deref() {
                    input(type="hidden", name="id", value=id);
                }

                @ if edit {
                    @ if let Some(id) = id.as_deref() {
                        a(href=format!("?id={}", id), class="pure-button"){ : "Cancel" }
                        : " ";
                    } else {
                        a(href="javascript:history.back()", class="pure-button"){ : "Cancel" }
                        : " ";
                    }
                } else {
                    a(href="..",class="pure-button") { : "Up" }
                    : " ";
                }
                @ if edit {
                    @ if let Some(_id) = id.as_deref() {
                        button(type="submit", class="pure-button pure-button-primary", formaction=".", formmethod="post"){
                            : "Save"
                        }
                    } else {
                        button(type="submit", class="pure-button pure-button-primary", formaction=".", formmethod="post", name="_method", value="put"){
                            : "Save"
                        }
                    }
                    : " ";
                } else {
                    a(href="?edit=true", class="pure-button button-green"){ : "New" }
                    : " ";
                }
                @ if !edit {
                    input(type="hidden", name="edit", value="true");
                    button(type="submit", class="pure-button pure-button-primary", formaction=".", formmethod="get"){
                        : "Edit"
                    }
                    : " ";
                    button(type="submit", class="pure-button button-warning", formaction=".", formmethod="post", name="_method", value="delete", onclick="return confirm('Are you sure?');"){
                        : "Delete"
                    }
                }
            }
            : subform
        }
    }
}

pub fn page_view(page_state: PageState) -> impl RenderOnce {
    let menu = menu(page_state.clone(), None);
    let page = page_state.page.expect("always some");
    let page_html = page.html.clone();
    owned_html! {
        : menu;
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
