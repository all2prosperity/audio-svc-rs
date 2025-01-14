use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RoleResponse {
    pub code: i32,
    pub msg: String,
    pub payload: Option<RolePayload>,
}

#[derive(Serialize)]
pub struct RolePayload {
    pub data: Vec<RoleInfo>,
    pub len: usize,
}

#[derive(Serialize)]
pub struct RoleInfo {
    pub id: String,
    pub name: String,
    pub picture_url: String,
    pub voice_id: String,
    pub audition_url: String,
}

#[derive(Deserialize)]
pub struct SwitchRoleRequest {
    pub role_id: String,
}

#[derive(Serialize)]
pub struct CommonResponse {
    pub code: i32,
    pub msg: String,
}

impl RoleResponse {
    pub fn success(roles: Vec<RoleInfo>) -> Self {
        let len = roles.len();
        Self {
            code: 0,
            msg: "ok".to_string(),
            payload: Some(RolePayload { data: roles, len }),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            code: -1,
            msg: msg.to_string(),
            payload: None,
        }
    }
}

impl CommonResponse {
    pub fn success() -> Self {
        Self {
            code: 0,
            msg: "ok".to_string(),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            code: -1,
            msg: msg.to_string(),
        }
    }
}


#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub desc: String,
    pub prompt: String, 
    pub my_story: String,
    pub voice_id: String,
    pub preference: String,
}

#[derive(Serialize)]
pub struct CreateRolePayload {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub prompt: String,
    pub my_story: String, 
    pub voice_id: String,
    pub preference: String,
}

#[derive(Serialize)]
pub struct CreateRoleResponse {
    pub code: i32,
    pub msg: String,
    pub payload: Option<CreateRolePayload>,
}

impl CreateRoleResponse {
    pub fn success(payload: CreateRolePayload) -> Self {
        Self {
            code: 0,
            msg: "ok".to_string(),
            payload: Some(payload),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            code: -1,
            msg: msg.to_string(),
            payload: None,
        }
    }
}