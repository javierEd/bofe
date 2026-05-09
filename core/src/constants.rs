use std::sync::LazyLock;

use regex::Regex;

pub const CACHE_PREFIX_GET_APPLICATION_BY_TOKEN: &str = "get_application_by_token";
pub const CACHE_PREFIX_GET_BOARD_BY_ID: &str = "get_board_by_id";
pub const CACHE_PREFIX_GET_BOARD_BY_SLUG: &str = "get_board_by_slug";
pub const CACHE_PREFIX_GET_SESSION_BY_ID: &str = "get_session_by_id";
pub const CACHE_PREFIX_GET_SESSION_BY_TOKEN: &str = "get_session_by_token";
pub const CACHE_PREFIX_GET_USER_BY_ID: &str = "get_user_by_id";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME: &str = "get_user_by_username";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL: &str = "get_user_by_username_or_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_EMAIL: &str = "get_user_id_by_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_USERNAME: &str = "get_user_id_by_username";

pub static REGEX_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[[:alnum:]]+(?:-[[:alnum:]]+)*\z").unwrap());
pub static REGEX_USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[-_.]?([[:alnum:]]+[-_.]?)+\z").unwrap());
