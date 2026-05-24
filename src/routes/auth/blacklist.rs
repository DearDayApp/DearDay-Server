use axum::extract::FromRef;
use moka::future::Cache;
use uuid::Uuid;

use crate::state::AppState;

use super::jwt::REFRESH_TTL;

/// In-memory revoked-token store. Single instance per process.
///
/// On restart, the blacklist is empty and revoked tokens become valid again
/// until they naturally expire. With access TTL = 30min this window is small
/// enough to accept; for stricter guarantees, swap moka for Redis later.
#[derive(Clone)]
pub struct Blacklist {
    cache: Cache<Uuid, ()>,
}

impl Blacklist {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(100_000)
                .time_to_live(
                    REFRESH_TTL
                        .to_std()
                        .expect("REFRESH_TTL fits in std::time::Duration"),
                )
                .build(),
        }
    }

    pub async fn revoke(&self, jti: Uuid) {
        self.cache.insert(jti, ()).await;
    }

    pub async fn is_revoked(&self, jti: Uuid) -> bool {
        self.cache.contains_key(&jti)
    }
}

impl Default for Blacklist {
    fn default() -> Self {
        Self::new()
    }
}

impl FromRef<AppState> for Blacklist {
    fn from_ref(state: &AppState) -> Self {
        state.blacklist.clone()
    }
}
