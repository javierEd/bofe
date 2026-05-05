use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum, Copy, Eq))]
#[derive(sqlx::Type, Clone, Deserialize, Serialize, PartialEq)]
#[sqlx(type_name = "board_visibility", rename_all = "lowercase")]
pub enum BoardVisibility {
    Private,
    Users,
    Public,
}
