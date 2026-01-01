use crate::application::administrators::create::{
    CreateAdministratorRequest, CreateAdministratorUseCase,
};
use crate::application::administrators::delete::DeleteAdministratorUseCase;
use crate::application::administrators::get::GetAdministratorUseCase;
use crate::application::administrators::list::{
    ListAdministratorsRequest, ListAdministratorsUseCase,
};
use crate::application::administrators::update::{
    UpdateAdministratorRequest, UpdateAdministratorUseCase,
};
use crate::domain::administrators::Administrator;
use crate::domain::password::PasswordService;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::administrators::PostgresAdministratorRepository;
use crate::presentation::handlers::auth::AuthUser;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::{JsonApiMeta, JsonApiResource, JsonApiResponse};
use crate::shared::validation::ValidatedJson;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdministratorResource {
    pub id: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub contact_number: Option<String>,
    pub email: String,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String)]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String)]
    pub updated_at: time::OffsetDateTime,
}

impl From<Administrator> for AdministratorResource {
    fn from(admin: Administrator) -> Self {
        Self {
            id: admin.id.to_string(),
            first_name: admin.first_name,
            middle_name: admin.middle_name,
            last_name: admin.last_name,
            suffix: admin.suffix,
            contact_number: admin.contact_number,
            email: admin.email,
            created_at: admin.created_at,
            updated_at: admin.updated_at,
        }
    }
}

/// Create a new administrator
#[utoipa::path(
    post,
    path = "/api/v1/admin/administrators",
    request_body = CreateAdministratorRequest,
    responses(
        (status = 201, description = "Administrator created successfully", body = JsonApiResponse<JsonApiResource<AdministratorResource>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "administrators"
)]
pub async fn create_admin(
    State(pool): State<DbPool>,
    _auth: AuthUser,
    ValidatedJson(req): ValidatedJson<CreateAdministratorRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresAdministratorRepository::new(pool));
    let hasher = Arc::new(PasswordService::new());
    let use_case = CreateAdministratorUseCase::new(repo, hasher);

    let admin = use_case.execute(req).await?;
    let resource = JsonApiResource::new(
        "administrators",
        admin.id.to_string(),
        AdministratorResource::from(admin),
    );

    Ok((StatusCode::CREATED, Json(JsonApiResponse::new(resource))))
}

/// Get an administrator by ID
#[utoipa::path(
    get,
    path = "/api/v1/admin/administrators/{id}",
    params(
        ("id" = Uuid, Path, description = "Administrator ID")
    ),
    responses(
        (status = 200, description = "Administrator found", body = JsonApiResponse<JsonApiResource<AdministratorResource>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Administrator not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "administrators"
)]
pub async fn get_admin(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresAdministratorRepository::new(pool));
    let use_case = GetAdministratorUseCase::new(repo);

    let admin = use_case.execute(id).await?;

    match admin {
        Some(admin) => {
            let resource = JsonApiResource::new(
                "administrators",
                admin.id.to_string(),
                AdministratorResource::from(admin),
            );
            Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
        }
        None => Err(AppError::NotFound("Administrator not found".to_string())),
    }
}

/// List all administrators with pagination
#[utoipa::path(
    get,
    path = "/api/v1/admin/administrators",
    params(ListAdministratorsRequest),
    responses(
        (status = 200, description = "List of administrators", body = JsonApiResponse<Vec<JsonApiResource<AdministratorResource>>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "administrators"
)]
pub async fn list_admins(
    State(pool): State<DbPool>,
    uri: Uri,
    Query(req): Query<ListAdministratorsRequest>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresAdministratorRepository::new(pool));
    let use_case = ListAdministratorsUseCase::new(repo.clone());

    // Capture pagination values before moving req
    let page_number = req.page.number;
    let page_size = req.page.size;

    let admins = use_case.execute(req).await?;

    // Get total count for pagination
    let total = crate::domain::administrators::AdministratorRepository::count(&*repo)
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    let resources: Vec<JsonApiResource<AdministratorResource>> = admins
        .into_iter()
        .map(|admin| {
            JsonApiResource::new(
                "administrators",
                admin.id.to_string(),
                AdministratorResource::from(admin),
            )
        })
        .collect();

    // Calculate pagination metadata
    let meta = JsonApiMeta::new()
        .with_page(page_number)
        .with_per_page(page_size)
        .with_total(total);

    // Generate pagination links using the helper
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

/// Update an administrator
#[utoipa::path(
    put,
    path = "/api/v1/admin/administrators/{id}",
    params(
        ("id" = Uuid, Path, description = "Administrator ID")
    ),
    request_body = UpdateAdministratorRequest,
    responses(
        (status = 200, description = "Administrator updated successfully", body = JsonApiResponse<JsonApiResource<AdministratorResource>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Administrator not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "administrators"
)]
pub async fn update_admin(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateAdministratorRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresAdministratorRepository::new(pool));
    let hasher = Arc::new(PasswordService::new());
    let use_case = UpdateAdministratorUseCase::new(repo, hasher);

    let admin = use_case.execute(id, req).await?;
    let resource = JsonApiResource::new(
        "administrators",
        admin.id.to_string(),
        AdministratorResource::from(admin),
    );

    Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
}

/// Delete an administrator
#[utoipa::path(
    delete,
    path = "/api/v1/admin/administrators/{id}",
    params(
        ("id" = Uuid, Path, description = "Administrator ID")
    ),
    responses(
        (status = 200, description = "Administrator deleted successfully", body = JsonApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Administrator not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "administrators"
)]
pub async fn delete_admin(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresAdministratorRepository::new(pool));
    let use_case = DeleteAdministratorUseCase::new(repo);

    let deleted = use_case.execute(id).await?;

    if deleted {
        let meta = JsonApiMeta::new().with_extra(json!({ "deleted": true }));
        Ok((
            StatusCode::OK,
            Json(JsonApiResponse::new(json!(null)).with_meta(meta)),
        ))
    } else {
        Err(AppError::NotFound("Administrator not found".to_string()))
    }
}
