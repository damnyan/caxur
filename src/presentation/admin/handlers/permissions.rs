use crate::application::permissions::list::{ListPermissionsRequest, ListPermissionsUseCase};
use crate::domain::permissions::PermissionScope;
use crate::shared::error::AppError;
use crate::shared::query::Qs;
use crate::shared::response::{JsonApiMeta, JsonApiResource, JsonApiResponse};
use axum::{
    Json,
    http::{StatusCode, Uri},
    response::IntoResponse,
};
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
    params(ListPermissionsRequest),
    responses(
        (status = 200, description = "List of permissions", body = JsonApiResponse<Vec<JsonApiResource<PermissionResource>>>),
    ),
    tag = "Admin / Permission Management"
)]
pub async fn list_permissions(
    uri: Uri,
    Qs(req): Qs<ListPermissionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    let use_case = ListPermissionsUseCase::new().with_scope(PermissionScope::Administrator);

    // Capture pagination values before moving req
    let page_number = req.page.number;
    let page_size = req.page.size;

    let permissions = use_case.execute(req);
    let total = use_case.count();

    let resources: Vec<JsonApiResource<PermissionResource>> = permissions
        .into_iter()
        .map(|p| {
            JsonApiResource::new(
                "permissions",
                p.name.clone(),
                PermissionResource {
                    name: p.name,
                    description: p.description,
                },
            )
        })
        .collect();

    // Calculate pagination metadata
    let meta = JsonApiMeta::new()
        .with_page(page_number)
        .with_per_page(page_size)
        .with_total(total);

    // Generate pagination links
    let links = crate::shared::pagination::PaginationLinkBuilder::from_uri(
        &uri,
        page_number,
        page_size,
        total,
    )
    .build();

    Ok((
        StatusCode::OK,
        Json(
            JsonApiResponse::new(resources)
                .with_meta(meta)
                .with_links(links),
        ),
    ))
}
