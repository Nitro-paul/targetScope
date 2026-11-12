// doc/programs/web_server.md#admin-password

use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::Argon2;
use axum::http::{HeaderMap, StatusCode};

use crate::errors::ApiError;
use crate::state::AppState;

pub const TOKEN_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);
pub const MAX_WRONG_PER_MINUTE: usize = 5;
pub const COOKIE: &str = "targetscope_admin";
pub const SETTING: &str = "admin_password_hash";

#[derive(Default)]
pub struct AdminSessions {
    tokens: Mutex<HashMap<String, Instant>>,
    wrong: Mutex<HashMap<IpAddr, VecDeque<Instant>>>,
}

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "io_error", e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
}

// doc/programs/web_server.md#admin-password
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        let _ = f.read_exact(&mut bytes);
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|c| c.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE)
        .map(|(_, v)| v.to_string())
}

impl AdminSessions {
    pub fn login(&self) -> String {
        let token = new_token();
        self.tokens.lock().unwrap().insert(token.clone(), Instant::now());
        token
    }

    pub fn logout(&self, headers: &HeaderMap) {
        if let Some(t) = cookie_token(headers) {
            self.tokens.lock().unwrap().remove(&t);
        }
    }

    pub fn logout_all(&self) {
        self.tokens.lock().unwrap().clear();
    }

    pub fn logged_in(&self, headers: &HeaderMap) -> bool {
        let Some(t) = cookie_token(headers) else { return false };
        let mut tokens = self.tokens.lock().unwrap();
        match tokens.get_mut(&t) {
            Some(last) if last.elapsed() <= TOKEN_IDLE_TIMEOUT => {
                *last = Instant::now();
                true
            }
            Some(_) => {
                tokens.remove(&t);
                false
            }
            None => false,
        }
    }

    // doc/programs/web_server.md#admin-password
    pub fn check_attempt(&self, ip: IpAddr) -> Result<(), ApiError> {
        let mut wrong = self.wrong.lock().unwrap();
        let list = wrong.entry(ip).or_default();
        while list.front().is_some_and(|t| t.elapsed() > Duration::from_secs(60)) {
            list.pop_front();
        }
        if list.len() >= MAX_WRONG_PER_MINUTE {
            return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "too_many_attempts", "too many wrong passwords; wait a minute"));
        }
        Ok(())
    }

    pub fn wrong_attempt(&self, ip: IpAddr) {
        self.wrong.lock().unwrap().entry(ip).or_default().push_back(Instant::now());
    }
}

pub fn password_hash(st: &AppState) -> Option<String> {
    st.settings.get(SETTING).filter(|h| !h.is_empty())
}

// doc/programs/web_server.md#admin-password
pub fn require(st: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    if password_hash(st).is_none() || st.admin.logged_in(headers) {
        return Ok(());
    }
    Err(ApiError::new(StatusCode::UNAUTHORIZED, "auth_required", "the admin password is required"))
}

pub fn set_cookie(token: &str) -> String {
    format!("{COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/")
}

pub fn clear_cookie() -> String {
    format!("{COOKIE}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
}
