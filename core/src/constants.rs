use std::sync::LazyLock;

use regex::Regex;

pub const CACHE_PREFIX_GET_BOARD_BY_ID: &str = "get_board_by_id";
pub const CACHE_PREFIX_GET_BOARD_BY_SLUG: &str = "get_board_by_slug";
pub const CACHE_PREFIX_GET_IDENTITY_USER: &str = "get_identity_user";
pub const CACHE_PREFIX_GET_USER_BY_ID: &str = "get_user_by_id";
pub const CACHE_PREFIX_GET_USER_BY_IDENTITY_USER_ID: &str = "get_user_by_identity_user_id";

pub static REGEX_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[[:alnum:]]+(?:-[[:alnum:]]+)*\z").unwrap());
