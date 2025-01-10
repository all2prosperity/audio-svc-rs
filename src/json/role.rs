use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRoleRequest {
    pub name: String,
    pub prompt: String,
}
