use crate::application::permissions::list::ListPermissionsUseCase;
use crate::domain::permissions::PermissionScope;
use crate::shared::error::AppError;
use crate::shared::response::{JsonApiResource, JsonApiResponse};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResource {
    pub name: String,
    pub description: String,
}

/// List all available permissions
#[utoipa::path(
    get,
    path = "/api/v1/admin/permissions",
    responses(
        (status = 200, description = "List of permissions", body = JsonApiResponse<Vec<JsonApiResource<PermissionResource>>>),
    ),
    tag = "Admin / Permission Management"
)]
pub async fn list_permissions() -> Result<impl IntoResponse, AppError> {
    let use_case = ListPermissionsUseCase::new().with_scope(PermissionScope::Administrator);
    let permissions = use_case.execute();

    let resources: Vec<JsonApiResource<PermissionResource>> = permissions
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            JsonApiResource::new(
                "permissions",
                i.to_string(),
                PermissionResource {
                    name: p.name,
                    description: p.description,
                },
            )
        })
        .collect();

    Ok((StatusCode::OK, Json(JsonApiResponse::new(resources))))
}
