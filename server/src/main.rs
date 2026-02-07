use actix_web::{web, App, HttpServer};
use tracing_subscriber::EnvFilter;
use std::sync::Arc;

mod config;
mod crypto;
mod db;
mod handlers;
mod middleware;
mod models;

use config::Config;
use handlers::{create_link, get_share_page, unlock_link};
use middleware::cleanup_expired_links;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("shredlink_server=info".parse()?))
        .init();

    let config = Config::from_env()?;
    let pool = Arc::new(db::create_pool(&config).await?);
    db::run_migrations(&pool).await?;

    let pool_cleanup = pool.clone();
    let cleanup_interval_secs = config.cleanup_interval_secs;
    actix_web::rt::spawn(async move {
        cleanup_expired_links(pool_cleanup, cleanup_interval_secs).await;
    });

    let bind = format!("{}:{}", config.host, config.port);
    tracing::info!("Listening on {}", bind);

    let public_dir = std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            let p = cwd.join("public");
            if p.exists() { Some(p) } else { None }
        })
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("public")
        });
    let rate_limiter = middleware::CreateRateLimiter::new(config.create_rate_limit_per_minute);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(rate_limiter.clone()))
            .service(
                web::resource("/api/create").route(web::post().to(create_link)),
            )
            .service(web::scope("/api").route("/unlock/{token}", web::post().to(unlock_link)))
            .route("/s/{token}", web::get().to(get_share_page))
            .service(
                actix_files::Files::new("/", public_dir.clone()).index_file("index.html"),
            )
    })
    .bind(&bind)?
    .run()
    .await?;
    Ok(())
}
