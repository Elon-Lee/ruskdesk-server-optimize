use crate::database::{Database, LicenceKey};
use axum::{extract::{Path, Query, Extension, Form}, response::Html, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AdminState {
    pub db: Database,
}

#[derive(Debug, Deserialize)]
struct ListParams {
    offset: Option<i64>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct CreateKeyForm {
    key: Option<String>,
    duration: Option<String>, // one of: 1d,7d,1m,1q,1y
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExtendParams {
    option: String, // 1d,7d,1m,1q,1y
}

#[derive(Debug, Serialize)]
struct ListResponse {
    total: i64,
    items: Vec<LicenceKey>,
}

fn ttl_seconds_for_option(option: &str) -> i64 {
    match option {
        "1d" => 86400,
        "7d" => 86400 * 7,
        "1m" => 86400 * 30,
        "1q" => 86400 * 90,
        "1y" => 86400 * 365,
        _ => 86400 * 30,
    }
}

fn generate_default_key() -> String {
    let raw = uuid::Uuid::new_v4().simple().to_string();
    raw[..16].to_string()
}

async fn list_keys(Extension(state): Extension<AdminState>, Query(p): Query<ListParams>) -> Json<ListResponse> {
    let offset = p.offset.unwrap_or(0);
    let limit = p.limit.unwrap_or(20).clamp(1, 100);
    let total = state.db.count_keys().await.unwrap_or(0);
    let items = state.db.list_keys(offset, limit).await.unwrap_or_default();
    Json(ListResponse { total, items })
}

async fn create_key(Extension(state): Extension<AdminState>, Form(p): Form<CreateKeyForm>) -> Html<String> {
    let key = p.key.unwrap_or_else(generate_default_key);
    let seconds = ttl_seconds_for_option(p.duration.as_deref().unwrap_or("1m"));
    let expired_at = chrono::Utc::now().timestamp() + seconds;
    let _ = state.db.insert_key(&key, expired_at, true, p.note.as_deref()).await;
    Html(format!("<meta http-equiv=\"refresh\" content=\"0;url=/admin\"><p>Created key: {}</p>", key))
}

async fn extend_key(Extension(state): Extension<AdminState>, Path(key): Path<String>, Query(p): Query<ExtendParams>) -> Html<String> {
    let seconds = ttl_seconds_for_option(&p.option);
    let _ = state.db.extend_key_by(&key, seconds).await;
    Html("<meta http-equiv=\"refresh\" content=\"0;url=/admin\">".to_string())
}

async fn index_html() -> Html<String> {
    let html = r#"<!doctype html>
<html>
  <head>
    <meta charset='utf-8'/>
    <title>RustDesk Keys Admin</title>
    <style>
      body { font-family: -apple-system, Arial, sans-serif; margin: 24px; }
      table { border-collapse: collapse; width: 100%; margin-top: 16px; }
      th, td { border: 1px solid #ddd; padding: 8px; }
      th { background: #f5f5f5; text-align: left; }
      form.inline { display: inline; }
    </style>
  </head>
  <body>
    <h2>Keys Admin</h2>
    <form method='post' action='/api/keys'>
      <label>Key (optional 16 chars): <input name='key' maxlength='64' /></label>
      <label>Duration:
        <select name='duration'>
          <option value='1d'>1天</option>
          <option value='7d'>7天</option>
          <option value='1m' selected>1个月</option>
          <option value='1q'>1个季度</option>
          <option value='1y'>1年</option>
        </select>
      </label>
      <label>Note: <input name='note' maxlength='200' /></label>
      <button type='submit'>新增Key</button>
    </form>
    <div id='list'></div>
    <script>
      let offset = 0, limit = 20;
      async function load() {
        const res = await fetch(`/api/keys?offset=${offset}&limit=${limit}`);
        const data = await res.json();
        const rows = data.items.map(k => `
          <tr>
            <td>${k.licence_key}</td>
            <td>${new Date(k.registered_at*1000).toLocaleString()}</td>
            <td>${new Date(k.expired_at*1000).toLocaleString()}</td>
            <td>${k.active ? '有效' : '无效'}</td>
            <td>
              <a href='/api/keys/${k.licence_key}/extend?option=1d'>+1天</a>
              <a href='/api/keys/${k.licence_key}/extend?option=7d'>+7天</a>
              <a href='/api/keys/${k.licence_key}/extend?option=1m'>+1个月</a>
              <a href='/api/keys/${k.licence_key}/extend?option=1q'>+1季度</a>
              <a href='/api/keys/${k.licence_key}/extend?option=1y'>+1年</a>
            </td>
          </tr>`).join('');
        const totalPages = Math.ceil(data.total/limit);
        const cur = Math.floor(offset/limit)+1;
        document.getElementById('list').innerHTML = `
          <table>
            <thead><tr><th>Key</th><th>注册日期</th><th>过期日期</th><th>有效状态</th><th>操作</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>
          <div style='margin-top:12px'>
            <button ${offset<=0?'disabled':''} onclick='prev()'>上一页</button>
            <span>第 ${cur} / ${totalPages||1} 页</span>
            <button ${offset+limit>=data.total?'disabled':''} onclick='next()'>下一页</button>
          </div>`;
      }
      function prev(){ if(offset>0){ offset-=limit; load(); }}
      function next(){ offset+=limit; load(); }
      load();
    </script>
  </body>
<\/html>"#;
    Html(html.to_string())
}

pub async fn spawn_admin(db: Database, base_port: i32) {
    let state = AdminState { db };
    let app = Router::new()
        .route("/admin", get(index_html))
        .route("/api/keys", get(list_keys).post(create_key))
        .route("/api/keys/:key/extend", get(extend_key))
        .layer(axum::Extension(state));

    // Bind to localhost only
    let port = std::env::var("ADMIN_PORT")
        .ok()
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(base_port + 100);
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    hbb_common::tokio::spawn(async move {
        if let Err(e) = axum::Server::bind(&addr).serve(app.into_make_service()).await {
            hbb_common::log::error!("Admin server failed: {}", e);
        }
    });
    hbb_common::log::info!("Admin UI: http://{}/admin", addr);
}


