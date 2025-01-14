use std::time::SystemTime;

use axum::{
    extract::{Json, State},
    http::{HeaderMap, StatusCode},
    Extension,
};
use diesel::{associations::HasTable, SelectableHelper as _};

use crate::models::{
    role::Role,
    schema::{self, user_role},
};
use crate::structures::user::CurrentUser;
use crate::structures::{AppState, CommonResponse, RoleInfo, RoleResponse, SwitchRoleRequest};
use crate::{
    models::schema::roles::dsl::*,
    structures::{CreateRolePayload, CreateRoleRequest, CreateRoleResponse},
};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::SelectableHelper;

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
    let mut results = schema::roles::table
        .filter(schema::roles::is_default.eq(true))
        .select(Role::as_select())
        .load(conn)
        .expect("Error loading roles");

    let self_created_roles = schema::roles::table
        .filter(schema::roles::created_by.eq(user.user_id))
        .select(Role::as_select())
        .load(conn)
        .expect("Error loading roles");

    results.extend(self_created_roles);

    let resp_roles = results
        .iter()
        .map(|r| RoleInfo {
            id: r.id.clone(),
            name: r.name.clone(),
            picture_url: "http://example.com/pig.jpg".to_string(),
            voice_id: r.voice_id.clone(),
            audition_url: r.audition_url.clone(),
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

pub async fn create_role(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    headers: HeaderMap,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Json<CreateRoleResponse>, StatusCode> {
    // 验证 device_id
    if !headers.contains_key(DEVICE_ID_HEADER) {
        return Ok(Json(CreateRoleResponse::error("Missing device ID")));
    }

    let conn = &mut state
        .db_pool
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 创建新角色
    let role = Role {
        id: xid::new().to_string(),
        is_default: false,
        created_by: user.user_id,
        name: payload.name,
        picture_url: "".to_string(),
        voice_id: payload.voice_id,
        audition_url: "".to_string(),
        prompt: payload.prompt,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    };

    // 插入数据库
    match diesel::insert_into(schema::roles::table)
        .values(&role)
        .execute(conn)
    {
        Ok(_) => {
            let response_payload = CreateRolePayload {
                id: role.id,
                created_by: role.created_by,
                name: role.name,
                desc: payload.desc,
                prompt: role.prompt,
                my_story: payload.my_story,
                voice_id: role.voice_id,
                preference: payload.preference,
            };
            Ok(Json(CreateRoleResponse::success(response_payload)))
        }
        Err(_) => Ok(Json(CreateRoleResponse::error("Failed to create role"))),
    }
}
