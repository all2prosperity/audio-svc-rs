use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    Extension,
};
use diesel::SelectableHelper as _;

use crate::models::schema::roles::dsl::*;
use crate::models::{role::Role, schema::user_role};
use crate::structures::user::CurrentUser;
use crate::structures::{AppState, CommonResponse, RoleInfo, RoleResponse, SwitchRoleRequest};
use diesel::prelude::*;

const DEVICE_ID_HEADER: &str = "X-OZ-Device-ID";

pub async fn get_roles(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
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

    let results = roles
        .select(Role::as_select())
        .load(conn)
        .expect("Error loading roles");

    let resp_roles = results
        .iter()
        .map(|r| RoleInfo {
            id: r.id.clone(),
            name: r.name.clone(),
            picture_url: "http://example.com/pig.jpg".to_string(),
        })
        .collect();

    Ok(Json(RoleResponse::success(resp_roles)))
}

pub async fn switch_role(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    headers: HeaderMap,
    Json(payload): Json<SwitchRoleRequest>,
) -> Result<Json<CommonResponse>, StatusCode> {
    // 验证 device_id
    if !headers.contains_key(DEVICE_ID_HEADER) {
        return Ok(Json(CommonResponse::error("Missing device ID")));
    }

    let user_id = user.user_id;

    let conn = &mut state
        .db_pool
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // do a upsert
    let _ = diesel::insert_into(user_role::table)
        .values((
            user_role::id.eq(user_id),
            user_role::role_id.eq(&payload.role_id),
        ))
        .on_conflict((user_role::id, user_role::role_id))
        .do_update()
        .set(user_role::role_id.eq(&payload.role_id))
        .execute(conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CommonResponse::success()))
}
