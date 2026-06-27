use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use serde::{Deserialize, Serialize};

use crate::constants::REGEX_COLOR_CODE;

#[derive(sqlx::Type, Clone, Deserialize, Serialize)]
#[sqlx(transparent, type_name = "color_code")]
pub struct ColorCode(String);

impl From<String> for ColorCode {
    fn from(value: String) -> Self {
        ColorCode(value)
    }
}

#[Scalar]
impl ScalarType for ColorCode {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value
            && REGEX_COLOR_CODE.is_match(value)
        {
            Ok(ColorCode(value.clone()))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.clone())
    }
}
