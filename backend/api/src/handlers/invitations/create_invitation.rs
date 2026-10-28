use crate::{
    middlewares::auth_middleware::Authdata, services::email_service::EmailService, Arcadia,
};
use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        invitation::{Invitation, SentInvitation},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create invitation",
    tag = "Invitation",
    path = "/api/invitations",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully sent the invitation", body=Invitation),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    invitation: Json<SentInvitation>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // inviting an applicant accepts their application, which is a staff decision
    if invitation.user_application_id.is_some() {
        arc.pool
            .require_permission(user.sub, &UserPermission::UpdateUserApplication, req.path())
            .await?;
    }

    let current_user = arc.pool.find_user_with_id(user.sub).await?;
    if current_user.invitations <= 0 {
        return Err(Error::NoInvitationsAvailable);
    }

    let invitation_expiration_days = arc.settings.lock().unwrap().invitation_expiration_days;

    let created_invitation = arc
        .pool
        .create_invitation(&invitation, current_user.id, invitation_expiration_days)
        .await?;

    // Send invitation email
    if let Ok(email_service) = EmailService::new(&arc) {
        if let Err(e) = email_service
            .send_invitation_email(
                &invitation.receiver_email,
                &current_user.username,
                &created_invitation.invitation_key,
                &invitation.message,
                invitation_expiration_days,
            )
            .await
        {
            // Log the error but don't fail the invitation creation
            log::warn!(
                "Failed to send invitation email to {}: {}",
                invitation.receiver_email,
                e
            );
        }
    } else {
        log::warn!("Email service not configured, skipping invitation email");
    }

    Ok(HttpResponse::Created().json(created_invitation))
}
