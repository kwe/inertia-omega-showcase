//! An in-memory session store that forgets expired sessions.
//!
//! `tower_sessions::MemoryStore` skips expired sessions when loading them but
//! never removes them, so every visitor that triggers a flash message leaves a
//! session behind for good. This store prunes expired sessions whenever a new
//! one is created, so memory stays bounded by the number of active sessions.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use tower_sessions::cookie::time::OffsetDateTime;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{self, SessionStore};

#[derive(Clone, Debug, Default)]
pub struct PruningMemoryStore(Arc<Mutex<HashMap<Id, Record>>>);

impl PruningMemoryStore {
    fn sessions(&self) -> MutexGuard<'_, HashMap<Id, Record>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The number of sessions held, expired or not.
    pub fn len(&self) -> usize {
        self.sessions().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl SessionStore for PruningMemoryStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        let mut sessions = self.sessions();
        let now = OffsetDateTime::now_utc();
        sessions.retain(|_, session| session.expiry_date > now);
        while sessions.contains_key(&record.id) {
            record.id = Id::default();
        }
        sessions.insert(record.id, record.clone());
        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.sessions().insert(record.id, record.clone());
        Ok(())
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        let now = OffsetDateTime::now_utc();
        Ok(self
            .sessions()
            .get(id)
            .filter(|session| session.expiry_date > now)
            .cloned())
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        self.sessions().remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_sessions::cookie::time::Duration;

    fn record(expires_in: Duration) -> Record {
        Record {
            id: Id::default(),
            data: Default::default(),
            expiry_date: OffsetDateTime::now_utc() + expires_in,
        }
    }

    #[tokio::test]
    async fn creating_a_session_prunes_expired_ones() {
        let store = PruningMemoryStore::default();
        store.save(&record(Duration::minutes(-1))).await.unwrap();
        store.save(&record(Duration::minutes(-1))).await.unwrap();
        let active = record(Duration::minutes(5));
        store.save(&active).await.unwrap();
        assert_eq!(store.len(), 3);

        store
            .create(&mut record(Duration::minutes(5)))
            .await
            .unwrap();

        assert_eq!(store.len(), 2);
        assert!(store.load(&active.id).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn expired_sessions_are_not_loaded() {
        let store = PruningMemoryStore::default();
        let expired = record(Duration::minutes(-1));
        store.save(&expired).await.unwrap();

        assert!(store.load(&expired.id).await.unwrap().is_none());
    }
}
