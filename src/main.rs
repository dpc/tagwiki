//! tagwiki

use anyhow::Result;
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

fn warp_temporary_redirect_after_post(location: &str) -> warp::http::Response<&'static str> {
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
struct GetPrompt {
    edit: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct PostForm {
    body: String,
}

fn html_for_editing_page(page: &page::Parsed) -> String {
    format!(
        "<form action='.' method='POST'><textarea name='body'>{}</textarea><br/><input type=submit></form>",
        page.source_body
    )
}

fn path_to_tags(path: &FullPath) -> Vec<&str> {
    path.as_str()
        .split('/')
        .map(|t| t.trim())
        .filter(|t| t != &"")
        .collect()
}

async fn handle_post(
    state: Arc<State>,
    path: FullPath,
    form: PostForm,
) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    let tags = path_to_tags(&path);
    let mut write = state.page_store.write().await;
    let results = write.find(tags.as_slice());

    match results.matching_pages.len() {
        1 => {
            let page = write
                .get(results.matching_pages[0].clone())
                .await
                .map_err(|e| warp::reject::custom(RejectAnyhow(e)))?;

            let page = page.with_new_source_body(&get_rid_of_windows_newlines(form.body));

            write
                .put(&page)
                .await
                .map_err(|e| warp::reject::custom(RejectAnyhow(e)))?;

            Ok(Box::new(warp_temporary_redirect_after_post(".".into())))
        }
        _ => {
            // TODO: ERROR
            Ok(Box::new(format!("Results: {:?}", results)))
        }
    }
}

async fn handle_get(
    state: Arc<State>,
    path: FullPath,
    query: GetPrompt,
) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    let tags = path_to_tags(&path);
    let read = state.page_store.read().await;
    let results = read.find(tags.as_slice());
    if results.matching_tags != tags {
        return Ok(Box::new(warp_temporary_redirect(
            &("/".to_string() + &results.matching_tags.join("/")),
        )));
    }
    if results.matching_pages.len() == 1 {
        let page = read
            .get(results.matching_pages[0].clone())
            .await
            .map_err(|e| warp::reject::custom(RejectAnyhow(e)))?;
        Ok(Box::new(warp::reply::html(if query.edit.is_none() {
            page.html
                + "<form action='.' method='get'><input type='hidden' name='edit' value='true' /><button type='submit'/>Edit Page</form>"
        } else {
            html_for_editing_page(&page)
        })))
    } else {
        Ok(Box::new(format!("Results: {:?}", results)))
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
        .and(with_state(state.clone()))
        .and(warp::path::full())
        .and(warp::query::<GetPrompt>())
        .and(warp::get())
        .and_then(handle_get)
        .or(warp::any()
            .and(with_state(state))
            .and(warp::path::full())
            .and(warp::post())
            .and(warp::filters::body::form())
            .and_then(handle_post));
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
