use crate::application::users::list::{ListUsersRequest, ListUsersUseCase};
use crate::domain::users::UserRepository;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::presentation::dtos::UserResource;
use crate::presentation::extractors::AuthUser;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::{JsonApiMeta, JsonApiResource, JsonApiResponse};
use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use std::sync::Arc;

/// List all users with pagination
#[utoipa::path(
    get,
    path = "/api/v1/users", // Should this be /api/v1/admin/users? The original was /api/v1/users.
    // Admin module usually implies /admin prefix?
    // The nesting in router was: .nest("/api/v1/users", routes::users::routes())
    // which had list_users on "/". So it was /api/v1/users.
    // If I move it to admin, maybe I should keep the path?
    // The user request said "future group of user".
    // "admin" -> "handlers", "routes".
    // "admin/users" route?
    // If I want to keep existing API contract, `list_users` was at `/api/v1/users`.
    // But `create_user` (signup) and `get_user` (profile) were also there.
    // Segregating routes:
    // Admin routes:
    // /api/v1/admin/administrators
    // /api/v1/admin/roles
    // /api/v1/admin/permissions
    //
    // Users:
    // /api/v1/users (GET list - Admin only usually?)
    // /api/v1/users (POST create - Signup)
    // /api/v1/users/{id} (GET/PUT/DELETE)
    //
    // If we want to segregate, maybe:
    // presentation/admin/routes/users.rs -> Handles /api/v1/admin/users?
    // presentation/client/routes/users.rs -> Handles /api/v1/users?
    //
    // If I change the path of list_users to /api/v1/admin/users, I break the API.
    // But maybe that's improved? "aligned with the oas tagging".
    // Tag was "Admin / User Management".
    // I should probably keep `list_users` in admin module, but the route might be `/api/v1/users` if I mount it there?
    // No, if I separate modules, I usually mount them separately.
    // Router structure:
    // /api/v1/admin/users -> list_users
    // /api/v1/users -> create/get/update/delete?
    //
    // Steps to migrate `list_users` to `/api/v1/admin/users`:
    // 1. Move handler to admin/handlers/users.rs
    // 2. Register route in admin/routes/users.rs
    // 3. Mount admin/routes/users.rs at `/api/v1/admin/users` in router.rs.
    //
    // This seems to be the "maintainable" "segregated" way.
    // I will assume `list_users` moves to `/api/v1/admin/users`.
    //
    params(ListUsersRequest),
    responses(
        (status = 200, description = "List of users", body = JsonApiResponse<Vec<JsonApiResource<UserResource>>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin / User Management"
)]
pub async fn list_users(
    State(pool): State<DbPool>,
    uri: Uri,
    Query(req): Query<ListUsersRequest>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = ListUsersUseCase::new(repo.clone());

    // Capture pagination values before moving req
    let page_number = req.page.number;
    let page_size = req.page.size;

    let users = use_case.execute(req).await?;

    // Get total count for pagination
    let total = repo.count().await.map_err(AppError::InternalServerError)?;

    let resources: Vec<JsonApiResource<UserResource>> = users
        .into_iter()
        .map(|user| JsonApiResource::new("users", user.id.to_string(), UserResource::from(user)))
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
