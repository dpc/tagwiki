//! tagwiki

use anyhow::{bail, format_err, Result};
use log::info;
use std::sync::Arc;
use structopt::StructOpt;
use warp::{path::FullPath, Filter};

use serde_derive::Deserialize;

use page::StoreMut;

/// Command line options
mod cli;
/// Page
mod page;

mod index;

/// Utils
mod util;

use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use horrorshow::{box_html, owned_html};

#[derive(Debug)]
struct RejectAnyhow(anyhow::Error);

impl warp::reject::Reject for RejectAnyhow {}

struct State {
    page_store:
        Arc<tokio::sync::RwLock<index::Index<Box<dyn page::store::StoreMut + Sync + Send>>>>,
}

fn with_state(
    state: Arc<State>,
) -> impl Filter<Extract = (Arc<State>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

fn warp_temporary_redirect(location: &str) -> warp::http::Response<&'static str> {
    warp::http::Response::builder()
        .status(307)
        .header(warp::http::header::LOCATION, location)
        .body("")
        .expect("correct redirect")
}

fn warp_temporary_redirect_to_get_method(location: &str) -> warp::http::Response<&'static str> {
    warp::http::Response::builder()
        .status(303)
        .header(warp::http::header::LOCATION, location)
        .body("")
        .expect("correct redirect")
}

fn get_rid_of_windows_newlines(s: String) -> String {
    s.chars().filter(|ch| *ch != '\r').collect()
}

#[derive(Deserialize, Debug)]
struct GetParams {
    edit: Option<bool>,
    id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct PostForm {
    body: Option<String>,
    id: Option<String>,
    _method: Option<String>,
}

impl PostForm {
    fn get_body(&self) -> Result<&str> {
        self.body
            .as_deref()
            .ok_or_else(|| format_err!("Missing body"))
    }
}

fn render_html_page(body: impl RenderOnce) -> impl RenderOnce {
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

fn render_page_editing_view(page: Option<&page::Parsed>) -> impl RenderOnce {
    if let Some(page) = page.as_ref() {
        let body = page.source_body.clone();
        let id = page.id().to_owned();
        (box_html! {
            form(action=".", method="post") {
                input(type="submit", value="Save", class="pure-button pure-button-primary");
                input(type="hidden", name="id", value=id);
                textarea(name="body") {
                    : body
                }
            }
        }) as Box<dyn RenderBox>
    } else {
        box_html! {
            form(action=".", method="post") {
                input(type="submit", value="Save", class="pure-button pure-button-primary");
                input(type="hidden", name="_method", value="put");
                textarea(name="body");
            }
        }
    }
}

fn render_page_view(page: &page::Parsed) -> impl RenderOnce {
    let page_html = page.html.clone();
    let id = page.id().to_owned();
    let id_copy = id.clone();
    owned_html! {
        div(class="pure-menu pure-menu-horizontal") {
            form(action="..", method="get", class="pure-menu-item") {
                button(type="submit", class="pure-button"){
                    : "Up"
                }
            }
            form(action="/", method="get", class="pure-menu-item") {
                input(type="hidden", name="edit", value="true");
                button(type="submit", class="pure-button button-green"){
                    : "New"
                }
            }
            form(action=".", method="get", class="pure-menu-item") {
                input(type="hidden", name="edit", value="true");
                input(type="hidden", name="id", value=id);
                button(type="submit", class="pure-button pure-button-primary"){
                    : "Edit"
                }
            }
            form(action=".", method="post", class="pure-menu-item") {
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

fn render_post_list(posts: impl Iterator<Item = index::PageInfo> + 'static) -> impl RenderOnce {
    owned_html! {
        div(class="pure-menu pure-menu-horizontal") {
            form(action="..", method="get", class="pure-menu-item") {
                button(type="submit", class="pure-button"){
                    : "Up"
                }
            }
            form(action="/", method="get", class="pure-menu-item") {
                input(type="hidden", name="edit", value="true");
                button(type="submit", class="pure-button button-green"){
                    : "New"
                }
            }
        }
        ul {
            @ for post in posts {
                li {
                    a(href=format!("?id={}", post.id)) : post.title
                }
            }
        }
    }
}

fn warp_reply_from_render(render: impl RenderOnce) -> Box<dyn warp::Reply> {
    Box::new(warp::reply::html(
        render.into_string().expect("rendering without errors"),
    ))
}

fn path_to_tags(path: &FullPath) -> Vec<&str> {
    path.as_str()
        .split('/')
        .map(|t| t.trim())
        .filter(|t| t != &"")
        .collect()
}

async fn handle_style_css() -> std::result::Result<warp::http::Response<String>, warp::Rejection> {
    Ok(warp::http::Response::builder()
        .status(200)
        .header(warp::http::header::CONTENT_TYPE, "text/css")
        .body(
            // include_str!("../resources/reset.css").to_string()
            include_str!("../resources/style.css").to_string(),
        )
        .expect("correct redirect"))
}

async fn handle_post_wrapped(
    state: Arc<State>,
    path: FullPath,
    form: PostForm,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    if let Some("put") = form._method.as_deref() {
        // workaround for not being able to use `method="put"` in html forms
        handle_put(state, path, form)
            .await
            .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
    } else if let Some("delete") = form._method.as_deref() {
        handle_delete(state, path, form)
            .await
            .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
    } else {
        handle_post(state, path, form)
            .await
            .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
    }
}

async fn handle_post(
    state: Arc<State>,
    path: FullPath,
    form: PostForm,
) -> Result<Box<dyn warp::Reply>> {
    let tags = path_to_tags(&path);
    let mut write = state.page_store.write().await;

    let post_id = if let Some(id) = form.id.as_deref() {
        id.to_owned()
    } else {
        let results = write.find(tags.as_slice());
        match results.matching_pages.len() {
            1 => results.matching_pages[0].id.clone(),
            0 => bail!("Page not found"),
            _ => return Ok(Box::new(warp_temporary_redirect_to_get_method(".".into()))),
        }
    };
    let page = write.get(post_id.to_owned()).await?;

    let page = page.with_new_source_body(&get_rid_of_windows_newlines(form.get_body()?.to_owned()));

    write.put(&page).await?;

    Ok(Box::new(warp_temporary_redirect_to_get_method(&format!(
        "?id={}",
        post_id
    ))))
}

async fn handle_put_wrapped(
    state: Arc<State>,
    path: FullPath,
    form: PostForm,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    handle_put(state, path, form)
        .await
        .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
}
async fn handle_put(
    state: Arc<State>,
    _path: FullPath,
    form: PostForm,
) -> Result<Box<dyn warp::Reply>> {
    let page = page::Parsed::new(&get_rid_of_windows_newlines(form.get_body()?.to_owned()));
    let mut write = state.page_store.write().await;
    write.put(&page).await?;

    Ok(Box::new(warp_temporary_redirect_to_get_method(&format!(
        "?id={}",
        page.id()
    ))))
}

async fn handle_delete_wrapped(
    state: Arc<State>,
    path: FullPath,
    form: PostForm,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    handle_delete(state, path, form)
        .await
        .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
}
async fn handle_delete(
    state: Arc<State>,
    _path: FullPath,
    query: PostForm,
) -> Result<Box<dyn warp::Reply>> {
    let mut write = state.page_store.write().await;
    let page = write
        .get(query.id.ok_or_else(|| format_err!("Missing ID"))?)
        .await?;
    write.delete(page.id().to_owned()).await?;

    Ok(Box::new(warp_temporary_redirect_to_get_method(&format!(
        ".",
    ))))
}

// I wish this could be generic
async fn handle_get_wrapped(
    state: Arc<State>,
    path: FullPath,
    query: GetParams,
) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    handle_get(state, path, query)
        .await
        .map_err(|e| warp::reject::custom(RejectAnyhow(e)))
}

fn render_page(page: Option<&page::Parsed>, edit: bool) -> Box<dyn RenderBox> {
    if edit {
        Box::new(render_page_editing_view(page)) as Box<dyn RenderBox>
    } else {
        Box::new(render_page_view(page.expect("always some"))) as Box<dyn RenderBox>
    }
}

async fn handle_get(
    state: Arc<State>,
    path: FullPath,
    query: GetParams,
) -> Result<Box<dyn warp::Reply>> {
    let tags = path_to_tags(&path);
    let read = state.page_store.read().await;

    if let Some(q_id) = query.id {
        let page = read.get(q_id).await?;
        return Ok(warp_reply_from_render(render_html_page(render_page(
            Some(&page),
            query.edit.is_some(),
        ))));
    } else if query.edit.is_some() {
        return Ok(warp_reply_from_render(render_html_page(render_page(
            None, true,
        ))));
    }
    let results = read.find(tags.as_slice());
    if results.matching_tags != tags {
        return Ok(Box::new(warp_temporary_redirect(
            &("/".to_string() + &results.matching_tags.join("/")),
        )));
    }
    if results.matching_pages.len() == 1 {
        let page = read.get(results.matching_pages[0].id.clone()).await?;
        Ok(warp_reply_from_render(render_html_page(render_page(
            Some(&page),
            query.edit.is_some(),
        ))))
    } else {
        Ok(warp_reply_from_render(render_html_page(render_post_list(
            results.matching_pages.into_iter(),
        ))))
    }
}

async fn start(opts: &cli::Opts) -> Result<()> {
    let state = Arc::new(State {
        page_store: Arc::new(tokio::sync::RwLock::new(
            index::Index::new(Box::new(page::store::FsStore::new(opts.path.clone())?)
                as Box<dyn page::store::StoreMut + Send + Sync>)
            .await?,
        )),
    });
    let handler = warp::any()
        .and(warp::path!("_style.css").and_then(handle_style_css))
        .or(with_state(state.clone())
            .and(warp::path::full())
            .and(warp::query::<GetParams>())
            .and(warp::get())
            .and_then(handle_get_wrapped))
        .or(with_state(state.clone())
            .and(warp::path::full())
            .and(warp::post())
            .and(warp::filters::body::form())
            .and_then(handle_post_wrapped))
        .or(with_state(state.clone())
            .and(warp::path::full())
            .and(warp::delete())
            .and(warp::filters::body::form())
            .and_then(handle_delete_wrapped))
        .or(with_state(state)
            .and(warp::path::full())
            .and(warp::put())
            .and(warp::filters::body::form())
            .and_then(handle_put_wrapped));
    info!("Listening on port {}", opts.port);
    let _serve = warp::serve(handler).run(([127, 0, 0, 1], opts.port)).await;

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = cli::Opts::from_args();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(start(&opts))?;

    Ok(())
}

/*
async fn handle_rejection(
    err: warp::Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    use warp::http::StatusCode;
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(DivideByZero) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "DIVIDE_BY_ZERO";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED";
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    Ok(warp::reply::with_status(message, code))
}*/
