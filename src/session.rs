use anyhow::Result;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};
use uuid::Uuid;

use crate::monitoring::MetacognitiveMonitor;

/// Session data containing monitor and metadata
#[derive(Clone)]
pub struct SessionData {
    pub monitor: Arc<Mutex<MetacognitiveMonitor>>,
    pub last_accessed: Instant,
    pub created_at: Instant,
}

/// Manages per-conversation sessions with automatic cleanup
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    ttl: Duration,
}

impl SessionManager {
    /// Create a new session manager with specified TTL
    pub fn new(ttl_minutes: u64) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_minutes * 60),
        }
    }

    /// Get or create a session, returning the session ID
    pub fn get_or_create_session(&self, session_id: Option<String>) -> String {
        let mut sessions = self.sessions.lock();

        let id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        if !sessions.contains_key(&id) {
            debug!("Creating new session: {}", id);
            let session_data = SessionData {
                monitor: Arc::new(Mutex::new(MetacognitiveMonitor::new())),
                last_accessed: Instant::now(),
                created_at: Instant::now(),
            };
            sessions.insert(id.clone(), session_data);
        } else {
            // Update last accessed time
            if let Some(session) = sessions.get_mut(&id) {
                session.last_accessed = Instant::now();
            }
        }

        id
    }

    /// Get the monitor for a specific session
    pub fn get_monitor(&self, session_id: &str) -> Result<Arc<Mutex<MetacognitiveMonitor>>> {
        let mut sessions = self.sessions.lock();

        match sessions.get_mut(session_id) {
            Some(session) => {
                session.last_accessed = Instant::now();
                Ok(session.monitor.clone())
            }
            None => {
                // Auto-create if missing
                debug!("Session {} not found, creating new one", session_id);
                let session_data = SessionData {
                    monitor: Arc::new(Mutex::new(MetacognitiveMonitor::new())),
                    last_accessed: Instant::now(),
                    created_at: Instant::now(),
                };
                let monitor = session_data.monitor.clone();
                sessions.insert(session_id.to_string(), session_data);
                Ok(monitor)
            }
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> usize {
        let mut sessions = self.sessions.lock();
        let now = Instant::now();
        let initial_count = sessions.len();

        sessions.retain(|id, session| {
            let age = now.duration_since(session.last_accessed);
            if age > self.ttl {
                debug!("Removing expired session: {} (age: {:?})", id, age);
                false
            } else {
                true
            }
        });

        let removed = initial_count - sessions.len();
        if removed > 0 {
            info!("Cleaned up {} expired sessions", removed);
        }
        removed
    }

    /// Get current session count
    pub fn session_count(&self) -> usize {
        self.sessions.lock().len()
    }

    /// Get session statistics
    pub fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.lock();
        let now = Instant::now();

        let mut oldest_age = Duration::from_secs(0);
        let mut total_age = Duration::from_secs(0);

        for session in sessions.values() {
            let age = now.duration_since(session.created_at);
            total_age += age;
            if age > oldest_age {
                oldest_age = age;
            }
        }

        let count = sessions.len();
        let avg_age = if count > 0 {
            total_age / count as u32
        } else {
            Duration::from_secs(0)
        };

        SessionStats {
            total_sessions: count,
            oldest_session_age: oldest_age,
            average_session_age: avg_age,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub oldest_session_age: Duration,
    pub average_session_age: Duration,
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            ttl: self.ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new(30);

        // Auto-generate session ID
        let id1 = manager.get_or_create_session(None);
        assert!(!id1.is_empty());

        // Use provided session ID
        let id2 = manager.get_or_create_session(Some("test-session".to_string()));
        assert_eq!(id2, "test-session");

        // Verify sessions exist
        assert_eq!(manager.session_count(), 2);
    }

    #[test]
    fn test_session_isolation() {
        let manager = SessionManager::new(30);

        let id1 = manager.get_or_create_session(None);
        let id2 = manager.get_or_create_session(None);

        let monitor1 = manager.get_monitor(&id1).unwrap();
        let monitor2 = manager.get_monitor(&id2).unwrap();

        // Verify different monitor instances
        assert!(!Arc::ptr_eq(&monitor1, &monitor2));

        // Test isolation by analyzing thoughts
        {
            let mut m1 = monitor1.lock();
            let _signals1 = m1.analyze_thought("test thought 1", 1);
        }

        {
            let mut m2 = monitor2.lock();
            let _signals2 = m2.analyze_thought("test thought 2", 1);
        }

        // Verify monitors are separate instances
        assert!(!Arc::ptr_eq(&monitor1, &monitor2));
    }

    #[test]
    fn test_session_cleanup() {
        let manager = SessionManager::new(0); // 0 minutes TTL for testing

        // Create sessions
        let _id1 = manager.get_or_create_session(None);
        let _id2 = manager.get_or_create_session(None);
        assert_eq!(manager.session_count(), 2);

        // Wait a bit
        thread::sleep(Duration::from_millis(100));

        // Clean up
        let removed = manager.cleanup_expired_sessions();
        assert_eq!(removed, 2);
        assert_eq!(manager.session_count(), 0);
    }
}
