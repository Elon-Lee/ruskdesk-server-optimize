use crate::database::{Database, LicenceKey};
use axum::{
    extract::{Path, Query, Extension, Form},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use headers::authorization::{Authorization, Basic};
use headers::HeaderMapExt;
// removed duplicate import of StatusCode
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
    key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateKeyForm {
    key: Option<String>,
    duration: Option<String>, // e.g. 10d,2w,3m,1y,permanent
    note: Option<String>,
    max_bind_ids: Option<i32>,
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

async fn auth_middleware<B>(req: Request<B>, next: Next<B>) -> Response {
    let user = std::env::var("ADMIN_USER").unwrap_or_else(|_| "elonlee".to_string());
    let pass = std::env::var("ADMIN_PASS").unwrap_or_else(|_| "Yiner520@".to_string());
    let unauthorized = || {
        (StatusCode::UNAUTHORIZED,
         [(header::WWW_AUTHENTICATE, header::HeaderValue::from_static("Basic realm=\"Admin\""))],
         "Unauthorized").into_response()
    };
    if let Some(Authorization(basic)) = req.headers().typed_get::<Authorization<Basic>>() {
        let u = basic.username();
        let p = basic.password();
        if u == user && p == pass {
            return next.run(req).await;
        }
    }
    unauthorized()
}

fn ttl_seconds_for_option(option: &str) -> Option<i64> {
    let opt = option.trim().to_lowercase();
    if opt == "permanent" || opt == "永久" || opt == "forever" {
        return None;
    }
    // pattern: <num><unit> where unit in d,w,m,y
    let (num_part, unit_part) = opt.split_at(opt.len().saturating_sub(1));
    let n: i64 = num_part.parse().unwrap_or(0).max(0);
    let seconds = match unit_part {
        "d" => 86400 * n,
        "w" => 86400 * 7 * n,
        "m" => 86400 * 30 * n,
        "y" => 86400 * 365 * n,
        _ => 0,
    };
    Some(seconds.max(0))
}

fn generate_default_key() -> String {
    // 32位 UUID（无中划线）
    uuid::Uuid::new_v4().simple().to_string()
}

fn is_valid_key_format(s: &str) -> bool {
    let ok_len = s.len() == 32;
    ok_len && s.chars().all(|c| c.is_ascii_hexdigit())
}

async fn list_keys(Extension(state): Extension<AdminState>, Query(p): Query<ListParams>) -> Json<ListResponse> {
    if let Some(k) = p.key.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let mut items = vec![];
        if let Ok(Some(rec)) = state.db.get_key(k).await {
            items.push(rec);
        }
        return Json(ListResponse { total: items.len() as i64, items });
    }
    let offset = p.offset.unwrap_or(0);
    let limit = p.limit.unwrap_or(20).clamp(1, 100);
    let total = state.db.count_keys().await.unwrap_or(0);
    let items = state.db.list_keys(offset, limit).await.unwrap_or_default();
    Json(ListResponse { total, items })
}

async fn create_key(Extension(state): Extension<AdminState>, Form(p): Form<CreateKeyForm>) -> Html<String> {
    let mut key = p.key.unwrap_or_else(generate_default_key);
    if key.trim().is_empty() {
        return Html("<p style='color:red'>Key不能为空</p>".to_string());
    }
    if !is_valid_key_format(&key) {
        return Html("<p style='color:red'>Key必须为32位UUID（无中划线）</p>".to_string());
    }
    if state.db.key_exists(&key).await.unwrap_or(false) {
        return Html("<p style='color:red'>Key已存在</p>".to_string());
    }
    let expired_at = match p.duration.as_deref() {
        Some(d) => match ttl_seconds_for_option(d) {
            Some(sec) => chrono::Utc::now().timestamp() + sec,
            None => i64::MAX / 2, // 近似永久
        },
        None => chrono::Utc::now().timestamp() + 86400 * 30,
    };
    let max_bind = p.max_bind_ids.unwrap_or(3).clamp(1, 1000);
    let _ = state.db.insert_key(&key, expired_at, true, p.note.as_deref(), max_bind).await;
    Html(format!("<meta http-equiv=\"refresh\" content=\"0;url=/admin\"><p>Created key: {}</p>", key))
}

async fn set_key_max_bind(Extension(state): Extension<AdminState>, Path((key, n)): Path<(String, i32)>) -> Html<String> {
    let n = n.clamp(1, 1000);
    let _ = state.db.set_key_max_bind(&key, n).await;
    Html("<meta http-equiv=\"refresh\" content=\"0;url=/admin\">".to_string())
}

async fn extend_key(Extension(state): Extension<AdminState>, Path(key): Path<String>, Query(p): Query<ExtendParams>) -> Html<String> {
    if let Some(seconds) = ttl_seconds_for_option(&p.option) {
        let _ = state.db.extend_key_by(&key, seconds).await;
    } else {
        // 永久：设置为很大时间
        let _ = state.db.extend_key_by(&key, (i64::MAX/2) - chrono::Utc::now().timestamp()).await;
    }
    Html("<meta http-equiv=\"refresh\" content=\"0;url=/admin\">".to_string())
}

async fn set_key_active(Extension(state): Extension<AdminState>, Path((key, flag)): Path<(String, i32)>) -> Html<String> {
    let _ = state.db.set_key_active(&key, flag != 0).await;
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
    <div style='margin-bottom:12px;'>
      <input id='searchKey' placeholder='按Key查询（32位HEX）' style='width:260px' />
      <button type='button' onclick='searchByKey()'>查询</button>
      <button type='button' onclick='openCreate()'>新增</button>
    </div>

    <div id='createModal' style='display:none; position:fixed; left:0; top:0; right:0; bottom:0; background:rgba(0,0,0,.35);'>
      <div style='background:#fff; padding:16px; width:560px; margin:10% auto; box-shadow:0 8px 32px rgba(0,0,0,.2)'>
        <h3>创建新Key</h3>
        <form id='createForm' method='post' action='/api/keys' onsubmit='return submitCreate(event)'>
          <div style='margin-bottom:8px'>
            <label>Key（32位UUID，无-）: <input id='key' name='key' maxlength='32' pattern='[0-9a-fA-F]{32}' /></label>
            <button type='button' onclick='gen()'>生成</button>
          </div>
          <div style='margin-bottom:8px'>
            <label>时长: <input id='duration' name='duration' placeholder='如: 10d/2w/3m/1y/permanent' /></label>
          </div>
          <div style='margin-bottom:8px'>
            <label>备注: <input name='note' maxlength='200' /></label>
          </div>
          <div style='margin-bottom:8px'>
            <label>最大可绑定ID数: <input type='number' min='1' max='1000' name='max_bind_ids' value='3' /></label>
          </div>
          <div>
            <button type='submit'>创建</button>
            <button type='button' onclick='closeCreate()'>取消</button>
          </div>
        </form>
      </div>
    </div>
    <div id='list'></div>
    <script>
      function gen(){ fetch('/api/keys/generate').then(r=>r.text()).then(t=>{ document.getElementById('key').value = t; }); }
      let offset = 0, limit = 20; let currentKey = '';
      async function load() {
        const q = currentKey ? `&key=${encodeURIComponent(currentKey)}` : '';
        const res = await fetch(`/api/keys?offset=${offset}&limit=${limit}${q}`);
        const data = await res.json();
        const rows = data.items.map(k => `
          <tr>
            <td>${k.licence_key}</td>
            <td>${new Date(k.registered_at*1000).toLocaleString()}</td>
            <td>${new Date(k.expired_at*1000).toLocaleString()}</td>
            <td>${k.active ? '有效' : '无效'}</td>
            <td>
              <span>${k.max_bind_ids}</span>
              <button class='inline' onclick='changeMax("${k.licence_key}", ${Math.max(1,(k.max_bind_ids||3)-1)})'>-</button>
              <button class='inline' onclick='changeMax("${k.licence_key}", ${(k.max_bind_ids||3)+1})'>+</button>
            </td>
            <td>
              <a href='/api/keys/${k.licence_key}/extend?option=1d'>+1天</a>
              <a href='/api/keys/${k.licence_key}/extend?option=7d'>+7天</a>
              <a href='/api/keys/${k.licence_key}/extend?option=1m'>+1个月</a>
              <a href='/api/keys/${k.licence_key}/extend?option=1y'>+1年</a>
              <a href='/api/keys/${k.licence_key}/extend?option=permanent'>永久</a>
              ${k.active ? `<a href='/api/keys/${k.licence_key}/active/0' style='color:#c00'>作废</a>` : `<a href='/api/keys/${k.licence_key}/active/1'>启用</a>`}
            </td>
          </tr>`).join('');
        const totalPages = currentKey ? 1 : Math.ceil(data.total/limit);
        const cur = Math.floor(offset/limit)+1;
        document.getElementById('list').innerHTML = `
          <table>
            <thead><tr><th>Key</th><th>注册日期</th><th>过期日期</th><th>有效状态</th><th>最大绑定数</th><th>操作</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>
          <div style='margin-top:12px'>
            ${currentKey ? '' : `<button ${offset<=0?'disabled':''} onclick='prev()'>上一页</button>`}
            ${currentKey ? '' : `<span>第 ${cur} / ${totalPages||1} 页</span>`}
            ${currentKey ? '' : `<button ${offset+limit>=data.total?'disabled':''} onclick='next()'>下一页</button>`}
          </div>`;
      }
      function prev(){ if(offset>0){ offset-=limit; load(); }}
      function next(){ offset+=limit; load(); }
      function searchByKey(){ currentKey = document.getElementById('searchKey').value.trim(); offset = 0; load(); }
      function openCreate(){ document.getElementById('createModal').style.display='block'; }
      function closeCreate(){ document.getElementById('createModal').style.display='none'; }
      async function submitCreate(e){ e.preventDefault(); const f = document.getElementById('createForm'); const body = new URLSearchParams(new FormData(f)); const res = await fetch('/api/keys', { method:'POST', headers: { 'Content-Type': 'application/x-www-form-urlencoded;charset=UTF-8' }, body }); await res.text(); closeCreate(); load(); return false; }
      async function changeMax(key, n){ await fetch(`/api/keys/${key}/max/${n}`); load(); }
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
        .route("/api/keys/generate", get(|| async { generate_default_key() }))
        .route("/api/keys/:key/extend", get(extend_key))
        .route("/api/keys/:key/max/:n", get(set_key_max_bind))
        .route("/api/keys/:key/active/:flag", get(set_key_active))
        .layer(middleware::from_fn(auth_middleware))
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


