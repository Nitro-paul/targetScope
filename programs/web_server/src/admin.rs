//! Optional admin password (doc/programs/web_server.md#admin-password): Argon2id hash in the
//! setting admin_password_hash; login token (256 bit) in memory, cookie HttpOnly SameSite=Strict,
//! expires after 30 minutes without use; at most 5 wrong passwords per minute per client.

use std::time::Duration;

pub const TOKEN_IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60);
pub const MAX_WRONG_PER_MINUTE: u32 = 5;
pub const COOKIE: &str = "targetscope_admin";

pub struct AdminSessions {
    // TODO: tokens with last use, wrong attempts per client address
}

pub fn hash_password(password: &str) -> String {
    let _ = password;
    todo!()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let _ = (password, hash);
    todo!()
}

// TODO: axum middleware `require_admin` for the protected endpoints (401 auth_required)
