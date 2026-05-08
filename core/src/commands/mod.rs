use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use bytesize::ByteSize;

mod application_commands;
mod board_commands;
mod card_commands;
mod list_commands;
mod user_commands;

pub use application_commands::*;
pub(crate) use board_commands::*;
pub(crate) use card_commands::*;
pub(crate) use list_commands::*;
pub use user_commands::*;

use crate::config::STORAGE_CONFIG;

#[allow(dead_code)]
fn get_available_space() -> ByteSize {
    let stats = uucore::fsext::statfs(STORAGE_CONFIG.path.as_os_str()).expect("Could not get storage stats");

    ByteSize(stats.f_bavail * stats.f_bsize as u64)
}

fn encrypt_password(value: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(value.as_bytes(), &salt).unwrap().to_string()
}
