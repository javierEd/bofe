use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use bytesize::ByteSize;
use validator::ValidationErrors;

mod application_commands;
mod board_commands;
mod card_commands;
mod list_commands;
mod session_commands;
mod user_commands;

pub use application_commands::*;
pub(crate) use board_commands::*;
pub(crate) use card_commands::*;
pub(crate) use list_commands::*;
pub use session_commands::*;
pub use user_commands::*;

use crate::config::STORAGE_CONFIG;

type ValidationResult<T = ()> = Result<T, ValidationErrors>;

trait OrValidationErrors<T> {
    fn or_validation_errors(self) -> ValidationResult<T>;
}

impl<T> OrValidationErrors<T> for Option<T> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.ok_or_else(Default::default)
    }
}

impl<T, E> OrValidationErrors<T> for Result<T, E> {
    fn or_validation_errors(self) -> ValidationResult<T> {
        self.map_err(|_| Default::default())
    }
}

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

pub(crate) fn verify_password(encrypted_password: &str, password: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(password_hash) = PasswordHash::new(encrypted_password) else {
        return false;
    };

    argon2.verify_password(password.as_bytes(), &password_hash).is_ok()
}
