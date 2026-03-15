use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::Reflect;

use crate::models::{Entry, SearchResponse, Stats};

/// Invoke a Tauri IPC command. Handles both success and error (rejected promise) cases.
async fn invoke<T: serde::de::DeserializeOwned>(
    cmd: &str,
    args: impl Serialize,
) -> Result<T, String> {
    let args_js = to_value(&args).map_err(|e| e.to_string())?;

    let window = web_sys::window().ok_or("no window")?;
    let tauri = Reflect::get(&window, &"__TAURI__".into()).map_err(|_| "Tauri not found")?;
    let core = Reflect::get(&tauri, &"core".into()).map_err(|_| "Tauri core not found")?;
    let invoke_fn = Reflect::get(&core, &"invoke".into()).map_err(|_| "invoke not found")?;
    let invoke_fn: js_sys::Function = invoke_fn.dyn_into().map_err(|_| "invoke not a function")?;

    let promise = invoke_fn
        .call2(&core, &cmd.into(), &args_js)
        .map_err(|e| format!("invoke call failed: {e:?}"))?;

    let result = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;

    from_value(result).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct ListEntriesArgs {
    #[serde(rename = "entryType", skip_serializing_if = "Option::is_none")]
    entry_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
}

pub async fn list_entries(
    entry_type: Option<String>,
    technology: Option<String>,
    tags: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchResponse, String> {
    invoke(
        "list_entries",
        ListEntriesArgs {
            entry_type,
            technology,
            tags,
            limit,
            offset,
        },
    )
    .await
}

#[derive(Serialize)]
struct SearchArgs {
    query: String,
    #[serde(rename = "entryType", skip_serializing_if = "Option::is_none")]
    entry_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    technology: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
}

pub async fn search_entries(
    query: String,
    entry_type: Option<String>,
    technology: Option<String>,
    limit: Option<u32>,
) -> Result<SearchResponse, String> {
    invoke(
        "search_entries",
        SearchArgs {
            query,
            entry_type,
            technology,
            limit,
        },
    )
    .await
}

#[derive(Serialize)]
struct GetEntryArgs {
    id: i64,
}

pub async fn get_entry(id: i64) -> Result<Entry, String> {
    invoke("get_entry", GetEntryArgs { id }).await
}

pub async fn list_technologies() -> Result<Vec<String>, String> {
    invoke("list_technologies", serde_json::json!({})).await
}

pub async fn list_tags() -> Result<Vec<String>, String> {
    invoke("list_tags", serde_json::json!({})).await
}

pub async fn fetch_stats() -> Result<Stats, String> {
    invoke("stats", serde_json::json!({})).await
}
