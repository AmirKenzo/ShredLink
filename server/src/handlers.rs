use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use nanoid::nanoid;

use std::sync::Arc;

use crate::config::Config;
use crate::crypto::{decrypt_text, encrypt_text, hash_password, verify_password};
use crate::db::DbPool;
use crate::middleware;
use crate::models::{CreateRequest, CreateResponse, ErrorResponse, LinkRow, UnlockRequest, UnlockResponse};

const TOKEN_LEN: usize = 16;

pub async fn create_link(
    pool: web::Data<Arc<DbPool>>,
    config: web::Data<Config>,
    rate_limiter: web::Data<middleware::CreateRateLimiter>,
    http_req: HttpRequest,
    req: web::Json<CreateRequest>,
) -> HttpResponse {
    if let Some(ip) = middleware::peer_ip(&http_req) {
        if !rate_limiter.check(&ip) {
            return HttpResponse::TooManyRequests().json(ErrorResponse {
                error: "Too many requests. Try again later.".to_string(),
            });
        }
    }
    if req.text.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "text is required".to_string(),
        });
    }
    if req.text.len() > config.max_text_size_bytes {
        return HttpResponse::PayloadTooLarge().json(ErrorResponse {
            error: format!("text exceeds max size ({} bytes)", config.max_text_size_bytes),
        });
    }

    let expire_minutes = req.expire_minutes.unwrap_or(0);
    let expire_hours = req.expire_hours.unwrap_or(0);
    let total_mins = expire_minutes as i64 + expire_hours as i64 * 60;
    let expires_at = if total_mins > 0 {
        Some(Utc::now() + Duration::minutes(total_mins))
    } else {
        None
    };

    let password_hash = match &req.password {
        Some(p) if !p.is_empty() => {
            let h = match hash_password(p) {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("hash_password: {}", e);
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "server error".to_string(),
                    });
                }
            };
            Some(h)
        }
        _ => None,
    };

    let encrypted_text = match encrypt_text(&req.text, &config.encryption_key_base64) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("encrypt: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "server error".to_string(),
            });
        }
    };

    let token = nanoid!(TOKEN_LEN);
    let one_time_view = if req.one_time_view { 1 } else { 0 };
    let one_time_password = if req.one_time_password { 1 } else { 0 };
    let expires_at_str = expires_at.map(|t| t.to_rfc3339());

    if let Err(e) = sqlx::query(
        "INSERT INTO links (token, encrypted_text, password_hash, expires_at, one_time_view, one_time_password) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&token)
    .bind(&encrypted_text)
    .bind(&password_hash)
    .bind(&expires_at_str)
    .bind(one_time_view)
    .bind(one_time_password)
    .execute((pool.get_ref()).as_ref())
    .await
    {
        tracing::warn!("insert: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: "server error".to_string(),
        });
    }

    let base = config.base_url.trim_end_matches('/');
    let url = format!("{}/s/{}", base, token);
    HttpResponse::Ok().json(CreateResponse { token, url })
}

pub async fn get_share_page(
    pool: web::Data<Arc<DbPool>>,
    config: web::Data<Config>,
    token: web::Path<String>,
) -> HttpResponse {
    let token = token.into_inner();
    let row = match sqlx::query_as::<_, LinkRow>(
        "SELECT id, token, encrypted_text, password_hash, expires_at, one_time_view, one_time_password, view_count, password_used, created_at FROM links WHERE token = ?",
    )
    .bind(&token)
    .fetch_optional((pool.get_ref()).as_ref())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return html_response(404, "Not found", "This link does not exist or has been removed.", "یافت نشد", "این لینک وجود ندارد یا حذف شده است.");
        }
        Err(e) => {
            tracing::warn!("fetch: {}", e);
            return html_response(500, "Error", "Something went wrong.", "خطا", "مشکلی پیش آمد.");
        }
    };

    if middleware::is_link_expired_or_consumed(&row) {
        return html_response(410, "Expired", "This link has expired or has already been used.", "منقضی شده", "این لینک منقضی شده یا قبلاً استفاده شده است.");
    }

    if let Some(ref hash) = row.password_hash {
        if hash.is_empty() {
            return show_decrypted(pool, config, row).await;
        }
        return HttpResponse::Found()
            .append_header(("Location", format!("/unlock.html?token={}", token)))
            .finish();
    }

    show_decrypted(pool, config, row).await
}

fn html_response(status: u16, title_en: &str, msg_en: &str, title_fa: &str, msg_fa: &str) -> HttpResponse {
    let body = format!(
        r#"<!DOCTYPE html><html lang="en" dir="ltr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ShredLink</title>
<script>
(function(){{
  var dark=localStorage.getItem('shredlink_dark')==='1'||(!localStorage.getItem('shredlink_dark')&&window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.classList.toggle('dark',!!dark);
  var lang=localStorage.getItem('shredlink_lang')||'en';
  document.documentElement.lang=lang==='fa'?'fa':'en';
  document.documentElement.dir=lang==='fa'?'rtl':'ltr';
  document.documentElement.setAttribute('data-lang',lang);
}})();
</script>
<script src="https://cdn.tailwindcss.com"></script><script>tailwind.config={{darkMode:'class'}}</script>
<link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,wght@0,400;0,600;0,700&family=Vazirmatn:wght@400;500;600;700&display=swap" rel="stylesheet"/>
<style>body{{font-family:'DM Sans',system-ui,sans-serif}}body.lang-fa{{font-family:'Vazirmatn','DM Sans',system-ui}}.lang-switcher .lang-btn.font-medium{{background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.06);color:#1e293b}}.dark .lang-switcher .lang-btn.font-medium{{background:#334155;box-shadow:0 1px 2px rgba(0,0,0,.2);color:#e2e8f0}}.lang-switcher .lang-btn:not(.font-medium){{background:transparent}}</style>
</head>
<body class="bg-slate-50 dark:bg-slate-900 min-h-screen text-slate-800 dark:text-slate-200 antialiased flex items-center justify-center p-4">
<div class="absolute top-4 right-4 flex items-center gap-2">
  <button type="button" id="dark-toggle" class="p-2 rounded-lg text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700" aria-label="Dark mode">
    <svg id="icon-sun" class="w-5 h-5 hidden dark:block" fill="currentColor" viewBox="0 0 20 20"><path d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"/></svg>
    <svg id="icon-moon" class="w-5 h-5 block dark:hidden" fill="currentColor" viewBox="0 0 20 20"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/></svg>
  </button>
  <div class="lang-switcher inline-flex rounded-xl border border-slate-200 dark:border-slate-600 overflow-hidden bg-slate-100/80 dark:bg-slate-700/50"><button type="button" id="lang-en" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50 font-medium">EN</button><button type="button" id="lang-fa" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50">فا</button></div>
</div>
<div class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-200 dark:border-slate-700 p-8 max-w-md w-full text-center">
  <h1 id="err-title" class="text-xl font-semibold text-slate-800 dark:text-slate-100 mb-2" data-en="{}" data-fa="{}">{}</h1>
  <p id="err-msg" class="text-slate-600 dark:text-slate-400 mb-4" data-en="{}" data-fa="{}">{}</p>
  <a href="/" id="err-back" class="mt-4 inline-block text-indigo-600 dark:text-indigo-400 hover:underline" data-en="Back home" data-fa="بازگشت به خانه">Back home</a>
</div>
<script>
(function(){{
  var lang=document.documentElement.getAttribute('data-lang')||'en';
  var L=lang==='fa'?'fa':'en';
  document.getElementById('err-title').textContent=document.getElementById('err-title').getAttribute('data-'+L);
  document.getElementById('err-msg').textContent=document.getElementById('err-msg').getAttribute('data-'+L);
  document.getElementById('err-back').textContent=document.getElementById('err-back').getAttribute('data-'+L);
  document.querySelectorAll('.lang-btn').forEach(function(el){{el.classList.toggle('font-medium',el.id==='lang-'+L);}});
  document.getElementById('dark-toggle').addEventListener('click',function(){{document.documentElement.classList.toggle('dark');localStorage.setItem('shredlink_dark',document.documentElement.classList.contains('dark')?'1':'0');}});
  document.getElementById('lang-en').addEventListener('click',function(){{localStorage.setItem('shredlink_lang','en');location.reload();}});
  document.getElementById('lang-fa').addEventListener('click',function(){{localStorage.setItem('shredlink_lang','fa');location.reload();}});
}})();
</script>
</body></html>"#,
        title_en, title_fa, title_en, msg_en, msg_fa, msg_en
    );
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR))
        .content_type("text/html; charset=utf-8")
        .body(body)
}

async fn show_decrypted(
    pool: web::Data<Arc<DbPool>>,
    config: web::Data<Config>,
    mut row: LinkRow,
) -> HttpResponse {
    let text = match decrypt_text(&row.encrypted_text, &config.encryption_key_base64) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("decrypt: {}", e);
            return html_response(500, "Error", "Could not decrypt content.", "خطا", "رمزگشایی محتوا ممکن نشد.");
        }
    };

    let new_view_count = row.view_count + 1;
    let _ = sqlx::query("UPDATE links SET view_count = ? WHERE id = ?")
        .bind(new_view_count)
        .bind(row.id)
        .execute((pool.get_ref()).as_ref())
        .await;
    row.view_count = new_view_count;

    let display = html_escape(&text);
    let raw_json = serde_json::to_string(&text).unwrap_or_default();
    let body = content_view_html(&display, &raw_json);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn content_view_html(display_content: &str, raw_content_json: &str) -> String {
    let safe_json = raw_content_json.replace("</script>", "<\\/script>");
    format!(
        r#"<!DOCTYPE html><html lang="en" dir="ltr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ShredLink – Content</title>
<script src="https://cdn.tailwindcss.com"></script><script>tailwind.config={{darkMode:'class'}}</script>
<link href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,wght@0,400;0,500;0,600;0,700&family=Vazirmatn:wght@400;500;600;700&display=swap" rel="stylesheet"/>
<style>body{{font-family:'DM Sans',system-ui,sans-serif}} body.lang-fa{{font-family:'Vazirmatn','DM Sans',system-ui}} .content-display{{unicode-bidi:plaintext;text-align:start}} .lang-switcher .lang-btn.font-medium{{background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.06);color:#1e293b}}.dark .lang-switcher .lang-btn.font-medium{{background:#334155;box-shadow:0 1px 2px rgba(0,0,0,.2);color:#e2e8f0}}.lang-switcher .lang-btn:not(.font-medium){{background:transparent}}</style>
</head><body class="bg-slate-50 dark:bg-slate-900 min-h-screen text-slate-800 dark:text-slate-200 antialiased transition-colors">
<div class="max-w-3xl mx-auto px-4 py-8">
<header class="flex items-center justify-between mb-6">
  <a href="/" class="text-indigo-600 dark:text-indigo-400 hover:underline font-medium" id="back-home">Back home</a>
  <div class="flex items-center gap-3">
    <button type="button" id="dark-toggle" class="p-2 rounded-lg text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-700" aria-label="Dark mode"><svg id="icon-sun" class="w-5 h-5 hidden dark:block" fill="currentColor" viewBox="0 0 20 20"><path d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"/></svg><svg id="icon-moon" class="w-5 h-5 block dark:hidden" fill="currentColor" viewBox="0 0 20 20"><path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"/></svg></button>
    <div class="lang-switcher inline-flex rounded-xl border border-slate-200 dark:border-slate-600 overflow-hidden bg-slate-100/80 dark:bg-slate-700/50"><button type="button" id="lang-en" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50 font-medium">EN</button><button type="button" id="lang-fa" class="lang-btn min-h-[44px] min-w-[52px] px-4 py-2.5 text-sm transition-colors text-slate-500 dark:text-slate-400 hover:bg-slate-200/70 dark:hover:bg-slate-600/50">فا</button></div>
  </div>
</header>
<main class="bg-white dark:bg-slate-800 rounded-2xl shadow-sm border border-slate-200 dark:border-slate-700 overflow-hidden">
  <div class="p-6 sm:p-8">
    <pre id="content-display" dir="auto" class="content-display whitespace-pre-wrap break-words text-slate-800 dark:text-slate-100 text-sm leading-relaxed font-sans max-h-[70vh] overflow-y-auto">{}</pre>
  </div>
  <div class="px-6 sm:px-8 pb-6 flex flex-wrap items-center gap-3 border-t border-slate-200 dark:border-slate-700 pt-4">
    <button type="button" id="copy-all" class="px-4 py-2.5 bg-indigo-600 hover:bg-indigo-700 dark:bg-indigo-500 dark:hover:bg-indigo-600 text-white rounded-xl font-medium text-sm transition-colors" data-copy="Copy" data-copied="Copied!">Copy all</button>
    <a href="/" class="text-sm text-slate-600 dark:text-slate-400 hover:underline" id="back-home-2">Back home</a>
  </div>
</main>
</div>
<script type="application/json" id="content-payload">{}</script>
<script>
(function(){{
  var dark = localStorage.getItem('shredlink_dark')==='1' || (!localStorage.getItem('shredlink_dark') && window.matchMedia('(prefers-color-scheme: dark)').matches);
  if(dark) document.documentElement.classList.add('dark'); else document.documentElement.classList.remove('dark');
  var lang = localStorage.getItem('shredlink_lang')||'en';
  document.documentElement.lang = lang==='fa'?'fa':'en'; document.documentElement.dir = lang==='fa'?'rtl':'ltr';
  document.body.classList.toggle('lang-fa', lang==='fa');
  var payloadEl = document.getElementById('content-payload');
  var rawText = '';
  try {{ if(payloadEl && payloadEl.textContent) rawText = JSON.parse(payloadEl.textContent); }} catch(e) {{}}
  function doCopy(){{
    var ta = document.createElement('textarea');
    ta.value = rawText;
    ta.style.position = 'fixed'; ta.style.left = '-9999px';
    document.body.appendChild(ta);
    ta.select();
    try {{ document.execCommand('copy'); }} catch(e) {{}}
    document.body.removeChild(ta);
  }}
  document.getElementById('copy-all').onclick = function(){{
    var btn = document.getElementById('copy-all');
    function showCopied(){{ btn.textContent = btn.getAttribute('data-copied') || 'Copied!'; setTimeout(function(){{ btn.textContent = btn.getAttribute('data-copy') || 'Copy all'; }}, 2000); }}
    if(navigator.clipboard && navigator.clipboard.writeText){{
      navigator.clipboard.writeText(rawText).then(showCopied, function(){{ doCopy(); showCopied(); }});
    }} else {{ doCopy(); showCopied(); }}
  }};
  document.getElementById('dark-toggle').onclick = function(){{
    document.documentElement.classList.toggle('dark');
    localStorage.setItem('shredlink_dark', document.documentElement.classList.contains('dark') ? '1' : '0');
  }};
  document.getElementById('lang-en').onclick = function(){{
    localStorage.setItem('shredlink_lang','en');
    document.documentElement.lang='en'; document.documentElement.dir='ltr'; document.body.classList.remove('lang-fa');
    document.getElementById('copy-all').setAttribute('data-copy','Copy'); document.getElementById('copy-all').setAttribute('data-copied','Copied!');
    document.getElementById('copy-all').textContent = 'Copy all';
    document.getElementById('back-home').textContent = document.getElementById('back-home-2').textContent = 'Back home';
  }};
  document.getElementById('lang-fa').onclick = function(){{
    localStorage.setItem('shredlink_lang','fa');
    document.documentElement.lang='fa'; document.documentElement.dir='rtl'; document.body.classList.add('lang-fa');
    document.getElementById('copy-all').setAttribute('data-copy','کپی'); document.getElementById('copy-all').setAttribute('data-copied','کپی شد!');
    document.getElementById('copy-all').textContent = 'کپی همه';
    document.getElementById('back-home').textContent = document.getElementById('back-home-2').textContent = 'بازگشت به خانه';
  }};
  if(lang==='fa'){{
    document.getElementById('copy-all').setAttribute('data-copy','کپی'); document.getElementById('copy-all').setAttribute('data-copied','کپی شد!');
    document.getElementById('copy-all').textContent = 'کپی همه';
    document.getElementById('back-home').textContent = document.getElementById('back-home-2').textContent = 'بازگشت به خانه';
    document.querySelectorAll('.lang-btn').forEach(function(el,i){{ el.classList.toggle('font-medium', el.id==='lang-fa'); }});
  }} else document.querySelectorAll('.lang-btn').forEach(function(el,i){{ el.classList.toggle('font-medium', el.id==='lang-en'); }});
}})();
</script>
</body></html>"#,
        display_content,
        safe_json
    )
}

pub async fn unlock_link(
    pool: web::Data<Arc<DbPool>>,
    config: web::Data<Config>,
    token: web::Path<String>,
    req: web::Json<UnlockRequest>,
) -> HttpResponse {
    let token = token.into_inner();
    let row = match sqlx::query_as::<_, LinkRow>(
        "SELECT id, token, encrypted_text, password_hash, expires_at, one_time_view, one_time_password, view_count, password_used, created_at FROM links WHERE token = ?",
    )
    .bind(&token)
    .fetch_optional((pool.get_ref()).as_ref())
    .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Link not found".to_string(),
            });
        }
        Err(e) => {
            tracing::warn!("fetch: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server error".to_string(),
            });
        }
    };

    if middleware::is_link_expired_or_consumed(&row) {
        return HttpResponse::Gone().json(ErrorResponse {
            error: "This link has expired or has already been used.".to_string(),
        });
    }

    let hash = match &row.password_hash {
        Some(h) if !h.is_empty() => h,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "This link is not password-protected.".to_string(),
            });
        }
    };

    if !verify_password(&req.password, hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Wrong password".to_string(),
        });
    }

    let text = match decrypt_text(&row.encrypted_text, &config.encryption_key_base64) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!("decrypt: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Server error".to_string(),
            });
        }
    };

    if row.one_time_password != 0 {
        let _ = sqlx::query("UPDATE links SET password_used = 1, view_count = view_count + 1 WHERE id = ?")
            .bind(row.id)
            .execute((pool.get_ref()).as_ref())
            .await;
    } else {
        let _ = sqlx::query("UPDATE links SET view_count = view_count + 1 WHERE id = ?")
            .bind(row.id)
            .execute((pool.get_ref()).as_ref())
            .await;
    }

    HttpResponse::Ok().json(UnlockResponse { text })
}

