use governor::{Quota, RateLimiter};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use crate::db::DbPool;
use crate::models::LinkRow;
use chrono::Utc;

#[derive(Clone)]
pub struct CreateRateLimiter(Arc<governor::DefaultKeyedRateLimiter<IpAddr>>);

impl CreateRateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        let rpm = NonZeroU32::new(requests_per_minute.max(1)).unwrap();
        let quota = Quota::per_minute(rpm);
        Self(Arc::new(RateLimiter::keyed(quota)))
    }

    pub fn check(&self, key: &IpAddr) -> bool {
        self.0.check_key(key).is_ok()
    }
}

pub fn peer_ip(req: &actix_web::HttpRequest) -> Option<IpAddr> {
    req.connection_info()
        .realip_remote_addr()
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
}

pub async fn cleanup_expired_links(pool: std::sync::Arc<DbPool>, interval_secs: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    interval.tick().await;
    loop {
        interval.tick().await;
        if let Err(e) = delete_expired_or_invalid(pool.as_ref()).await {
            tracing::warn!("cleanup error: {}", e);
        }
    }
}

async fn delete_expired_or_invalid(pool: &DbPool) -> anyhow::Result<u64> {
    let now = Utc::now();
    let r = sqlx::query(
        "DELETE FROM links WHERE expires_at IS NOT NULL AND datetime(expires_at) < datetime(?) \
         OR (one_time_view = 1 AND view_count > 0) \
         OR (one_time_password = 1 AND password_used = 1)",
    )
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    let deleted = r.rows_affected();
    if deleted > 0 {
        tracing::info!("cleanup deleted {} expired/invalid links", deleted);
    }
    Ok(deleted)
}

pub fn is_link_expired_or_consumed(row: &LinkRow) -> bool {
    if let Some(exp) = row.expires_at {
        if exp < Utc::now() {
            return true;
        }
    }
    if row.one_time_view != 0 && row.view_count > 0 {
        return true;
    }
    if row.one_time_password != 0 && row.password_used != 0 {
        return true;
    }
    false
}
