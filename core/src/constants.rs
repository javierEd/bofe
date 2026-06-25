use std::sync::LazyLock;

#[cfg(feature = "graphql")]
use std::borrow::Cow;

use regex::Regex;

#[cfg(feature = "graphql")]
use validator::ValidationError;

#[cfg(feature = "graphql")]
pub const BLACKLISTED_USERNAMES: [&str; 26] = [
    "admin",
    "api",
    "board",
    "boards",
    "card",
    "card",
    "edit",
    "email",
    "group",
    "groups",
    "list",
    "lists",
    "login",
    "logout",
    "member",
    "members",
    "new",
    "profile",
    "register",
    "reset-password",
    "reset_password",
    "reset.password",
    "search",
    "settings",
    "user",
    "users",
];

pub const CACHE_PREFIX_GET_ACTIVITY_BY_ID: &str = "get_activity_by_id";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_ALL_CARDS: &str = "get_all_cards";
pub const CACHE_PREFIX_GET_ALL_CARD_LABELS: &str = "get_all_card_labels";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_ALL_LABELS: &str = "get_all_labels";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_ALL_LISTS: &str = "get_all_lists";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_APPLICATION_BY_TOKEN: &str = "get_application_by_token";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_ATTACHMENT_BY_ID: &str = "get_attachment_by_id";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_ATTACHMENT_KEY_BY_ID: &str = "get_attachment_key_by_id";
#[cfg(feature = "graphql")]
pub const CACHE_PREFIX_GET_BLOB_BY_ID: &str = "get_blob_by_id";
pub const CACHE_PREFIX_GET_BOARD_BY_ID: &str = "get_board_by_id";
pub const CACHE_PREFIX_GET_BOARD_BY_SLUG: &str = "get_board_by_slug";
pub const CACHE_PREFIX_GET_BOARD_BY_USER_AND_SLUG: &str = "get_board_by_user_and_slug";
pub const CACHE_PREFIX_GET_LABEL_BY_ID: &str = "get_label_by_id";
pub const CACHE_PREFIX_GET_MEMBER: &str = "get_member";
pub const CACHE_PREFIX_GET_MEMBER_BY_ID: &str = "get_member_by_id";
pub const CACHE_PREFIX_GET_SESSION_BY_ID: &str = "get_session_by_id";
pub const CACHE_PREFIX_GET_SESSION_BY_TOKEN: &str = "get_session_by_token";
pub const CACHE_PREFIX_GET_USER_BY_ID: &str = "get_user_by_id";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME: &str = "get_user_by_username";
pub const CACHE_PREFIX_GET_USER_BY_USERNAME_OR_EMAIL: &str = "get_user_by_username_or_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_EMAIL: &str = "get_user_id_by_email";
pub const CACHE_PREFIX_GET_USER_ID_BY_USERNAME: &str = "get_user_id_by_username";

#[cfg(feature = "graphql")]
pub static ERROR_ALREADY_EXISTS: LazyLock<ValidationError> =
    LazyLock::new(|| ValidationError::new("already-exists").with_message(Cow::Borrowed("Already exists")));
#[cfg(feature = "graphql")]
pub static ERROR_CANT_BE_BLANK: LazyLock<ValidationError> =
    LazyLock::new(|| ValidationError::new("cant-be-blank").with_message(Cow::Borrowed("Can't be blank")));
#[cfg(feature = "graphql")]
pub static ERROR_IS_INVALID: LazyLock<ValidationError> =
    LazyLock::new(|| ValidationError::new("invalid").with_message(Cow::Borrowed("Is invalid")));
#[cfg(feature = "graphql")]
pub static ERROR_PASSWORD_MUST_CHANGE: LazyLock<ValidationError> = LazyLock::new(|| {
    ValidationError::new("password-must-change").with_message(Cow::Borrowed("Must be different from current password"))
});

#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_CONFIRM_EMAIL: &str = "failed-to-confirm-email";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_CONFIRM_PASSWORD_RESET: &str = "failed-to-confirm-password-reset";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_CREATE_SESSION: &str = "failed-to-create-session";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_CREATE_USER: &str = "failed-to-create-user";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_SEND_CONFIRMATION: &str = "failed-to-send-confirmation";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_UPDATE_EMAIL: &str = "failed-to-update-email";
#[cfg(feature = "graphql")]
pub const KEY_TEXT_FAILED_TO_UPLOAD_FILE: &str = "failed-to-upload-file";

#[cfg(feature = "graphql")]
pub static REGEX_COLOR_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A#[[:xdigit:]]{3,6}\z").unwrap());
#[cfg(feature = "graphql")]
pub static REGEX_SLUG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[[:alnum:]]+(?:-[[:alnum:]]+)*\z").unwrap());
#[cfg(feature = "graphql")]
pub static REGEX_USERNAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A[-_.]?([[:alnum:]]+[-_.]?)+\z").unwrap());

pub static STRIP_MARKDOWN_RULES: LazyLock<Vec<(Regex, &str)>> = LazyLock::new(|| {
    vec![
        // Headers (e.g., # Header) -> completely strip syntax
        (Regex::new(r"(?m)^#{1,6}\s+").unwrap(), ""),
        // Fenced code blocks (```rust ... ```) -> remove fences, keep code context
        (Regex::new(r"```[a-zA-Z]*\n?([\s\S]*?)```").unwrap(), "$1"),
        // Inline code blocks (`code`) -> remove backticks
        (Regex::new(r"`([^`]+)`").unwrap(), "$1"),
        // Bold / Italics (**text**, __text__, *text*, _text_) -> keep content
        (Regex::new(r"\*\*([^*]+)\*\*").unwrap(), "$1"),
        (Regex::new(r"__([^_]+)__").unwrap(), "$1"),
        (Regex::new(r"\*([^*]+)\*").unwrap(), "$1"),
        (Regex::new(r"_([^_]+)_").unwrap(), "$1"),
        // Images (![alt](url)) -> remove completely
        (Regex::new(r"!\[.*?\]\(.*?\)").unwrap(), ""),
        // Links ([text](url)) -> preserve link text only
        (Regex::new(r"\[(.*?)\]\(.*?\)").unwrap(), "$1"),
        // Blockquotes (e.g., > Quote) -> remove leading > arrow
        (Regex::new(r"(?m)^>\s+").unwrap(), ""),
        // Horizontal Rules (---, ***, ___) -> remove line completely
        (Regex::new(r"(?m)^[-*_]{3,}\s*$").unwrap(), ""),
    ]
});
