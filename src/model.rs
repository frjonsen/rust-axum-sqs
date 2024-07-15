use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Foo {
    pub id: Uuid,
    pub bar: i32,
    pub zar: String
}