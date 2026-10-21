use crate::{
    services::{
        auth_service::{
            validate_email, validate_password, validate_password_verification, validate_username,
        },
        email_service::EmailService,
        ip_check,
    },
    Arcadia,
};
use actix_web::{web, HttpRequest, HttpResponse};
use arcadia_common::error::{Error, Result};
use arcadia_shared::tracker::models::user::APIInsertUser;
use arcadia_storage::{
    models::{
        invitation::Invitation,
        user::{Register, User},
    },
    redis::RedisPoolInterface,
    sqlx::types::ipnetwork::IpNetwork,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterQuery {
    invitation_key: Option<String>,
}

#[utoipa::path(
    post,
    operation_id = "Register",
    tag = "Auth",
    path = "/api/auth/register",
    responses(
        (status = 200, description = "Successfully registered the user", body = User),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    new_user: web::Json<Register>,
    arc: web::Data<Arcadia<R>>,
    req: HttpRequest,
    query: web::Query<RegisterQuery>,
) -> Result<HttpResponse> {
    let mut invitation: Option<Invitation> = None;
    if !arc.settings.lock().unwrap().open_signups {
        let invitation_key = query
            .invitation_key
            .as_ref()
            .ok_or(Error::InvitationKeyRequired)?;

        invitation = Some(
            arc.pool
                .does_unexpired_invitation_exist(invitation_key)
                .await?,
        );

        // TODO: push check to db
        if invitation.as_ref().unwrap().receiver_id.is_some() {
            return Err(Error::InvitationKeyAlreadyUsed);
        }
    }

    validate_email(&new_user.email)?;
    validate_username(&new_user.username)?;
    validate_password(&new_user.password)?;
    validate_password_verification(&new_user.password, &new_user.password_verify)?;

    let client_ip = req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpNetwork>().ok())
        .unwrap();

    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(new_user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let arcadia_settings = arc.settings.lock().unwrap().clone();
    let user = arc
        .pool
        .create_user(
            &new_user,
            client_ip,
            &password_hash,
            &invitation,
            &arcadia_settings,
        )
        .await?;

    if arc.api.ip_check_providers.is_enabled() && ip_check::is_public(client_ip.ip()) {
        let ip_check_providers = arc.api.ip_check_providers.clone();
        let pool = arc.pool.clone();
        let http_client = arc.http_client.clone();
        let user_id = user.id;
        tokio::spawn(async move {
            let result = match ip_check_providers
                .check_ip(&http_client, client_ip.ip())
                .await
            {
                Ok(check) => {
                    pool.update_user_registration_ip_check(user_id, &check)
                        .await
                }
                Err(error) => Err(error),
            };
            if let Err(error) = result {
                log::warn!("Could not check the registration ip of user {user_id}: {error}");
            }
        });
    }

    // Mark existing announcements as read so they don't trigger notifications
    if let Err(e) = arc.pool.mark_all_announcements_as_read(user.id).await {
        log::warn!(
            "Failed to mark announcements as read for {}: {}",
            new_user.username,
            e
        );
    }

    // Inform tracker about the new user so it can accept announces immediately
    {
        let mut url = arc.tracker.url_internal.clone();
        url.path_segments_mut().unwrap().push("api").push("users");

        let payload = APIInsertUser {
            id: user.id as u32,
            passkey: user.passkey.parse().expect("invalid passkey format"),
            max_snatches_per_day: user.max_snatches_per_day.map(|x| x as u32),
        };

        // Fire and log; don't fail registration if tracker call fails
        let res = arc
            .internal_http_client
            .put(url)
            .header("x-api-key", arc.tracker.api_key.clone())
            .json(&payload)
            .send()
            .await;

        if res.is_err() {
            log::warn!("Tried to upsert user in tracker and got: {:?}", res);
        }
    }

    // Send welcome email
    if let Ok(email_service) = EmailService::new(&arc) {
        if let Err(e) = email_service
            .send_registration_email(&new_user.email, &new_user.username)
            .await
        {
            // Log the error but don't fail the registration
            log::warn!("Failed to send welcome email to {}: {}", new_user.email, e);
        }
    } else {
        log::warn!("Email service not configured, skipping welcome email");
    }

    // Send automated signup message if configured
    if let (Some(message), Some(sender_id), Some(conversation_name)) = (
        &arcadia_settings.automated_message_on_signup,
        arcadia_settings.automated_message_on_signup_sender_id,
        &arcadia_settings.automated_message_on_signup_conversation_name,
    ) {
        let locked = arcadia_settings
            .automated_message_on_signup_locked
            .unwrap_or(false);
        let user_url = arc
            .api
            .frontend_url
            .join(&format!("/user/{}", user.id))
            .unwrap();
        let username_link = format!("[url={}]{}[/url]", user_url.as_str(), user.username);
        let rendered_message = message.replace("{username}", &username_link);
        if let Err(e) = arc
            .pool
            .send_batch_messages(
                sender_id,
                &[user.id],
                conversation_name,
                &rendered_message,
                locked,
            )
            .await
        {
            log::error!(
                "Failed to send automated signup message to {}: {}",
                new_user.username,
                e
            );
        }
    }

    Ok(HttpResponse::Created().json(serde_json::json!(user)))
}
