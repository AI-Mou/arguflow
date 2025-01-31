use super::auth_handler::LoggedUser;
use crate::{
    data::models::{FileUploadCompletedNotificationWithName, Pool},
    errors::ServiceError,
    operators::notification_operator::{
        get_notifications_query, mark_all_notifications_as_read_query,
        mark_notification_as_read_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum Notification {
    FileUploadComplete(FileUploadCompletedNotificationWithName),
}

#[utoipa::path(
    get,
    path = "/notifications/{page}",
    context_path = "/api",
    tag = "notifications",
    responses(
        (status = 200, description = "Notifications for the user", body = [NotificationReturn]),
        (status = 400, description = "Service error relating to getting notifications for the user", body = [DefaultError]),
    ),
    params(
        ("page" = i64, description = "Page number of notifications to get"),
    ),
)]
pub async fn get_notifications(
    user: LoggedUser,
    page: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;

    let notifications =
        web::block(move || get_notifications_query(user_id, page.into_inner(), pool))
            .await?
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(notifications))
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct NotificationId {
    pub notification_id: uuid::Uuid,
}

#[utoipa::path(
    put,
    path = "/notifications",
    context_path = "/api",
    tag = "notifications",
    request_body(content = NotificationId, description = "JSON request payload with id of notification to mark read", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the notification is marked read"),
        (status = 400, description = "Service error relating to finding the notification and marking it read", body = [DefaultError]),
    ),
)]
pub async fn mark_notification_as_read(
    user: LoggedUser,
    notification_id: web::Json<NotificationId>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;

    web::block(move || {
        mark_notification_as_read_query(user_id, notification_id.into_inner().notification_id, pool)
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}

#[utoipa::path(
    put,
    path = "/notifications_readall",
    context_path = "/api",
    tag = "notifications",
    responses(
        (status = 204, description = "Confirmation that the all notification were marked read for the auth'ed user"),
        (status = 400, description = "Service error relating to finding the notifications for the auth'ed user and marking them read", body = [DefaultError]),
    ),
)]
pub async fn mark_all_notifications_as_read(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;

    web::block(move || mark_all_notifications_as_read_query(user_id, pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}
