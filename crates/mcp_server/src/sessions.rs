//! [DonEditor] MCP 세션과 요청 취소 플래그를 스레드 사이에서 안전하게 공유한다.

use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// 활성 요청이 없는 세션의 유휴 만료 시간이다.
const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(60 * 60);
/// 활성 세션을 밀어내지 않고 새 세션 생성을 거절하는 상한이다.
const MAX_SESSIONS: usize = 32;
/// 한 클라이언트가 등록할 수 있는 동시 요청 수이다.
const MAX_REQUESTS_PER_SESSION: usize = 32;
/// 모든 세션에 걸친 활성 요청 수를 제한한다.
const MAX_ACTIVE_REQUESTS: usize = 128;
/// 요청 ID의 직렬화 키가 과도한 메모리를 차지하지 않도록 제한한다.
const MAX_REQUEST_ID_BYTES: usize = 1024;

/// 개별 세션의 최근 활동 시각과 아직 완료되지 않은 요청이다.
struct Session {
    last_used: Instant,
    requests: HashMap<String, Arc<AtomicBool>>,
}

/// UI·HTTP·취소 통지가 같은 세션 상태를 공유하는 동기화 경계이다.
pub struct Sessions {
    sessions: Mutex<HashMap<String, Session>>,
}

impl Sessions {
    /// 세션이나 요청이 없는 레지스트리를 만든다.
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Mutex 중독 이후에도 취소·정리 경로가 다시 패닉하지 않게 상태를 복구한다.
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Session>> {
        self.sessions.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 요청 ID를 값의 JSON 표현으로 구분한다. 숫자 1과 문자열 "1"은 다른 키이다.
    fn request_key(id: &Value) -> Result<String, String> {
        let key = id.to_string();
        if key.len() > MAX_REQUEST_ID_BYTES {
            Err("MCP 요청 ID가 너무 깁니다".into())
        } else {
            Ok(key)
        }
    }

    /// 완료되지 않은 요청이 없는 유휴 세션만 정리한다. 활성·승인 대기 요청은 유지한다.
    fn expire_idle(sessions: &mut HashMap<String, Session>, now: Instant) {
        sessions.retain(|_, session| {
            !session.requests.is_empty() || now.saturating_duration_since(session.last_used) < SESSION_IDLE_TIMEOUT
        });
    }

    /// UUID 세션을 만든다. 만료 세션을 정리해도 상한에 도달하면 명시적으로 거절한다.
    pub fn create(&self) -> Result<String, String> {
        let now = Instant::now();
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, now);
        if sessions.len() >= MAX_SESSIONS {
            return Err("MCP 세션 수가 상한에 도달했습니다. 사용하지 않는 세션을 종료하세요".into());
        }
        let mut id = uuid::Uuid::new_v4().to_string();
        // 이미 발급된 UUID와 우연히 같을 때도 기존 세션을 덮어쓰지 않는다.
        while sessions.contains_key(&id) {
            id = uuid::Uuid::new_v4().to_string();
        }
        sessions.insert(
            id.clone(),
            Session {
                last_used: now,
                requests: HashMap::new(),
            },
        );
        Ok(id)
    }

    /// 만료를 반영하고 존재하는 세션의 활동 시각을 갱신한다.
    pub fn contains(&self, session: &str) -> bool {
        let now = Instant::now();
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, now);
        if let Some(session) = sessions.get_mut(session) {
            session.last_used = now;
            true
        } else {
            false
        }
    }

    /// 요청마다 독립적인 취소 플래그를 발급하며 같은 세션의 활성 ID 중복을 거절한다.
    pub fn register(&self, session: &str, id: &Value) -> Result<Arc<AtomicBool>, String> {
        let key = Self::request_key(id)?;
        let now = Instant::now();
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, now);
        let active: usize = sessions.values().map(|session| session.requests.len()).sum();
        let session = sessions.get_mut(session).ok_or("MCP 세션이 없거나 만료되었습니다")?;
        if session.requests.contains_key(&key) {
            return Err("같은 ID의 MCP 요청이 이미 실행 중입니다".into());
        }
        if session.requests.len() >= MAX_REQUESTS_PER_SESSION || active >= MAX_ACTIVE_REQUESTS {
            return Err("MCP 활성 요청 수가 상한에 도달했습니다".into());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        session.requests.insert(key, cancelled.clone());
        session.last_used = now;
        Ok(cancelled)
    }

    /// 같은 세션 안의 해당 요청만 취소한다. 알 수 없거나 완료된 ID는 무시한다.
    pub fn cancel(&self, session: &str, id: &Value) {
        let Ok(key) = Self::request_key(id) else { return };
        let now = Instant::now();
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, now);
        if let Some(session) = sessions.get_mut(session) {
            if let Some(cancelled) = session.requests.get(&key) {
                cancelled.store(true, Ordering::Release);
            }
            session.last_used = now;
        }
    }

    /// 완료된 요청의 ID를 재사용할 수 있도록 등록만 해제하고 취소 플래그는 바꾸지 않는다.
    pub fn finish(&self, session: &str, id: &Value) {
        let Ok(key) = Self::request_key(id) else { return };
        let now = Instant::now();
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, now);
        if let Some(session) = sessions.get_mut(session) {
            session.requests.remove(&key);
            session.last_used = now;
        }
    }

    /// 세션을 제거하고 현재 실행·대기 중인 요청 모두에 취소를 전달한다.
    pub fn remove(&self, session: &str) {
        let mut sessions = self.lock();
        Self::expire_idle(&mut sessions, Instant::now());
        if let Some(session) = sessions.remove(session) {
            // 레지스트리 밖의 UI 작업도 보관 중인 Arc로 취소 상태를 확인할 수 있다.
            for cancelled in session.requests.values() {
                cancelled.store(true, Ordering::Release);
            }
        }
    }

    /// 서버 중지 시 모든 요청을 취소하고 세션도 무효화한다.
    pub fn cancel_all(&self) {
        let mut sessions = self.lock();
        // 세션별 요청을 모두 취소한 뒤 메모리와 세션 식별자를 해제한다.
        for session in sessions.values() {
            for cancelled in session.requests.values() {
                cancelled.store(true, Ordering::Release);
            }
        }
        sessions.clear();
    }
}

impl Default for Sessions {
    /// 기본값도 빈 세션 레지스트리이다.
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Sessions {
    /// 마지막 레지스트리 소유자가 사라질 때도 남아 있는 작업에 취소를 전달한다.
    fn drop(&mut self) {
        self.cancel_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 같은 ID가 서로 다른 세션에 있더라도 취소가 섞이지 않아야 한다.
    #[test]
    fn cancellation_is_scoped_to_session_and_typed_id() {
        let sessions = Sessions::new();
        let first = sessions.create().unwrap();
        let second = sessions.create().unwrap();
        assert!(uuid::Uuid::parse_str(&first).is_ok());
        assert_ne!(first, second);
        let first_request = sessions.register(&first, &json!(1)).unwrap();
        let second_request = sessions.register(&second, &json!(1)).unwrap();
        let string_request = sessions.register(&first, &json!("1")).unwrap();
        sessions.cancel(&first, &json!(1));
        assert!(first_request.load(Ordering::Acquire));
        assert!(!second_request.load(Ordering::Acquire));
        assert!(!string_request.load(Ordering::Acquire));
        sessions.cancel("unknown", &json!(1));
        sessions.cancel(&first, &json!("missing"));
        assert!(!string_request.load(Ordering::Acquire));
    }

    /// 활성 ID는 중복을 거절하고 완료 후에는 새 취소 플래그로 다시 사용할 수 있다.
    #[test]
    fn duplicate_ids_are_rejected_until_finished() {
        let sessions = Sessions::new();
        let session = sessions.create().unwrap();
        let original = sessions.register(&session, &json!(7)).unwrap();
        assert!(sessions.register(&session, &json!(7)).is_err());
        sessions.finish(&session, &json!(7));
        assert!(!original.load(Ordering::Acquire));
        let next = sessions.register(&session, &json!(7)).unwrap();
        assert!(!Arc::ptr_eq(&original, &next));
        sessions.cancel(&session, &json!(7));
        assert!(next.load(Ordering::Acquire));
        assert!(!original.load(Ordering::Acquire));
        sessions.finish("unknown", &json!(7));
    }

    /// 세션 삭제와 전체 취소는 보관 중인 Arc에 전달되고 세션도 무효화한다.
    #[test]
    fn removal_and_shutdown_cancel_registered_requests() {
        let sessions = Sessions::new();
        let first = sessions.create().unwrap();
        let second = sessions.create().unwrap();
        let first_request = sessions.register(&first, &json!("a")).unwrap();
        let second_request = sessions.register(&second, &json!("b")).unwrap();
        sessions.remove(&first);
        assert!(first_request.load(Ordering::Acquire));
        assert!(!second_request.load(Ordering::Acquire));
        assert!(!sessions.contains(&first));
        assert!(sessions.register(&first, &json!("a")).is_err());
        assert!(sessions.contains(&second));
        sessions.cancel_all();
        assert!(second_request.load(Ordering::Acquire));
        assert!(!sessions.contains(&second));
        sessions.cancel_all();
        sessions.remove("unknown");
        assert!(sessions.create().is_ok());
    }

    /// 실제 대기 없이 검사 시각만 앞당겨 유휴 만료를 확인하고 활성 요청은 보호한다.
    #[test]
    fn idle_expiration_preserves_active_requests() {
        let sessions = Sessions::new();
        let idle = sessions.create().unwrap();
        let active = sessions.create().unwrap();
        let request = sessions.register(&active, &json!(1)).unwrap();
        {
            let mut state = sessions.lock();
            Sessions::expire_idle(
                &mut state,
                Instant::now() + SESSION_IDLE_TIMEOUT + Duration::from_secs(1),
            );
            assert!(!state.contains_key(&idle));
            assert!(state.contains_key(&active));
        }
        assert!(!request.load(Ordering::Acquire));
        sessions.finish(&active, &json!(1));
        let mut state = sessions.lock();
        Sessions::expire_idle(
            &mut state,
            Instant::now() + SESSION_IDLE_TIMEOUT + Duration::from_secs(1),
        );
        assert!(!state.contains_key(&active));
    }

    /// 세션 상한에서 활성 세션을 밀어내지 않고, 명시적 삭제 후에는 자리를 재사용한다.
    #[test]
    fn session_limit_preserves_existing_sessions() {
        let sessions = Sessions::new();
        let mut ids = Vec::new();
        // 상한까지 세션을 생성하고 마지막 세션에도 요청을 등록한다.
        for _ in 0..MAX_SESSIONS {
            ids.push(sessions.create().unwrap());
        }
        let active = sessions.register(&ids[0], &json!(1)).unwrap();
        assert!(sessions.create().is_err());
        assert!(sessions.contains(&ids[0]));
        assert!(!active.load(Ordering::Acquire));
        sessions.remove(&ids[1]);
        assert!(sessions.create().is_ok());
    }

    /// 세션별·전체 요청 상한 및 긴 ID를 거절하고 완료 후에는 용량을 되돌린다.
    #[test]
    fn request_limits_are_released_on_finish() {
        let sessions = Sessions::new();
        let first = sessions.create().unwrap();
        assert!(sessions.register("unknown", &json!(1)).is_err());
        assert!(
            sessions
                .register(&first, &json!("x".repeat(MAX_REQUEST_ID_BYTES)))
                .is_err()
        );
        // 첫 세션의 요청 상한까지 채운다.
        for id in 0..MAX_REQUESTS_PER_SESSION {
            sessions.register(&first, &json!(id)).unwrap();
        }
        assert!(sessions.register(&first, &json!("extra")).is_err());
        sessions.finish(&first, &json!(0));
        assert!(sessions.register(&first, &json!("replacement")).is_ok());
        // 다른 세션을 사용해 전체 요청 상한까지 채운다.
        for _ in 1..(MAX_ACTIVE_REQUESTS / MAX_REQUESTS_PER_SESSION) {
            let session = sessions.create().unwrap();
            for id in 0..MAX_REQUESTS_PER_SESSION {
                sessions.register(&session, &json!(id)).unwrap();
            }
        }
        let extra = sessions.create().unwrap();
        assert!(sessions.register(&extra, &json!(1)).is_err());
        sessions.finish(&first, &json!(1));
        assert!(sessions.register(&extra, &json!(1)).is_ok());
    }

    /// 여러 스레드가 같은 세션·ID를 동시에 등록해도 한 요청만 성공한다.
    #[test]
    fn concurrent_registration_rejects_duplicates() {
        let sessions = Arc::new(Sessions::new());
        let session = sessions.create().unwrap();
        let mut workers = Vec::new();
        // 레지스트리 Mutex가 동시 중복 검사를 직렬화하는지 확인한다.
        for _ in 0..8 {
            let sessions = sessions.clone();
            let session = session.clone();
            workers.push(std::thread::spawn(move || sessions.register(&session, &json!(1))));
        }
        let results: Vec<_> = workers.into_iter().map(|worker| worker.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    }

    /// 레지스트리를 보관한 마지막 소유자가 사라지면 외부 Arc도 취소 상태가 된다.
    #[test]
    fn dropping_registry_cancels_remaining_requests() {
        let request = {
            let sessions = Sessions::new();
            let session = sessions.create().unwrap();
            sessions.register(&session, &json!(1)).unwrap()
        };
        assert!(request.load(Ordering::Acquire));
    }
}
