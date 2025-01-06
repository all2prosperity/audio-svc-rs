use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
};

use crate::structures::{AppState, CommonResponse, RoleInfo, RoleResponse, SwitchRoleRequest};

const DEVICE_ID_HEADER: &str = "X-OZ-Device-ID";

pub async fn get_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RoleResponse>, StatusCode> {
    // 验证 device_id
    if !headers.contains_key(DEVICE_ID_HEADER) {
        return Ok(Json(RoleResponse::error("Missing device ID")));
    }

    // 从数据库获取角色列表
    let conn = &mut state
        .db_pool
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: 实现从数据库查询角色列表的逻辑
    let roles = vec![RoleInfo {
        role_id: "1".to_string(),
        role_name: "猪猪".to_string(),
        picture_url: "http://example.com/pig.jpg".to_string(),
    }];

    Ok(Json(RoleResponse::success(roles)))
}

pub async fn switch_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SwitchRoleRequest>,
) -> Result<Json<CommonResponse>, StatusCode> {
    // 验证 device_id
    if !headers.contains_key(DEVICE_ID_HEADER) {
        return Ok(Json(CommonResponse::error("Missing device ID")));
    }

    let device_id = headers
        .get(DEVICE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;

    // TODO: 实现切换角色的数据库操作

    Ok(Json(CommonResponse::success()))
}
