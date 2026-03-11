use loco_rs::prelude::*;
use axum::extract::Multipart;
use crate::models::{
    users,
};
use crate::models::_entities::seller_verifications::{self, ActiveModel, Entity};
use crate::models::_entities::sea_orm_active_enums::VerificationStatus;
use crate::views::seller_verification::SellerVerificationResponse;
use crate::controllers::products::UnauthorizedResponse;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateVerifyMultipart {
    #[schema(value_type = String, format = Binary)]
    pub face_photo: String,
    #[schema(value_type = String, format = Binary)]
    pub ktp_photo: String,
    #[schema(value_type = String, format = Binary)]
    pub face_ktp_photo: String,
    #[schema(value_type = String, format = Binary)]
    pub domicile_photo: String,
}

#[utoipa::path(
    post,
    path = "/api/seller/verify",
    tag = "sellers",
    request_body(content = CreateVerifyMultipart, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Verification data uploaded successfully", body = SellerVerificationResponse),
        (status = 401, description = "Unauthorized", body = UnauthorizedResponse),
        (status = 400, description = "Bad request"),
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn verify_seller(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    mut multipart: Multipart,
) -> Result<Response> {
    println!("================================================");
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await?;
    
    // Check if there is already a pending or approved verification
    let existing = Entity::find()
        .filter(seller_verifications::Column::UserId.eq(user.id))
        .one(&ctx.db)
        .await?;

    if let Some(v) = existing {
        if v.status == VerificationStatus::Pending || v.status == VerificationStatus::Approved {
            return Err(Error::BadRequest("Verification already exists or is pending".into()));
        }
    }

    let mut face_photo = None;
    let mut ktp_photo = None;
    let mut face_ktp_photo = None;
    let mut domicile_photo = None;
    let mut ktp_text = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        Error::BadRequest(format!("Multipart error: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("photo.jpg").to_string();
        let data = field.bytes().await.map_err(|e| {
            Error::BadRequest(format!("Byte error: {}", e))
        })?;

        if data.is_empty() {
            continue;
        }

        if name == "ktp_photo" {
            ktp_text = Some(extract_text_from_image(&data).await?);
        }

        let timestamp = chrono::Utc::now().timestamp();
        let extension = std::path::Path::new(&file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");
        
        let storage_filename = format!("{}_{}_{}.{}", user.id, name, timestamp, extension);
        let storage_path = std::path::PathBuf::from("seller_verifications")
            .join(user.id.to_string())
            .join(storage_filename);

        ctx.storage
            .as_ref()
            .upload(storage_path.as_path(), &data)
            .await?;

        let stored_path = storage_path.to_string_lossy().into_owned();

        match name.as_str() {
            "face_photo" => face_photo = Some(stored_path),
            "ktp_photo" => ktp_photo = Some(stored_path),
            "face_ktp_photo" => face_ktp_photo = Some(stored_path),
            "domicile_photo" => domicile_photo = Some(stored_path),
            _ => {}
        }
    }

    let face_photo = face_photo.ok_or_else(|| Error::BadRequest("face_photo is required".into()))?;
    let ktp_photo = ktp_photo.ok_or_else(|| Error::BadRequest("ktp_photo is required".into()))?;
    let face_ktp_photo = face_ktp_photo.ok_or_else(|| Error::BadRequest("face_ktp_photo is required".into()))?;
    let domicile_photo = domicile_photo.ok_or_else(|| Error::BadRequest("domicile_photo is required".into()))?;
    let ktp_text = ktp_text.unwrap_or_default();

    println!("ktp_text {}", ktp_text);

    let active_model = ActiveModel {
        user_id: Set(user.id),
        face_photo: Set(face_photo),
        ktp_photo: Set(ktp_photo),
        face_ktp_photo: Set(face_ktp_photo),
        domicile_photo: Set(domicile_photo),
        ktp_text: Set(Some(ktp_text)),
        status: Set(VerificationStatus::Pending),
        ..Default::default()
    };

    let model = active_model.insert(&ctx.db).await?;
    format::json(SellerVerificationResponse::new(&model))
}

async fn extract_text_from_image(image_data: &[u8]) -> Result<String> {
    let client = reqwest::Client::new();
    let base64_image = general_purpose::STANDARD.encode(image_data);
    let vllm_url = std::env::var("VLLM_URL").unwrap_or_else(|_| "http://localhost:11434/api/chat".to_string());

    let payload = serde_json::json!({
        "model": "Qwen/Qwen3.5-0.8B",
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "text",
                        "text": "Please extract and return all text visible in this ID card (KTP) image. Output only the text found."
                    },
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:image/jpeg;base64,{}", base64_image)
                        }
                    }
                ]
            }
        ]
    });

    let response = client
        .post(&vllm_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to call Qwen API: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(Error::BadRequest(format!("Qwen API error ({}): {}", status, body)));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to parse Qwen response: {}", e)))?;

    let text = result["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| Error::BadRequest("Invalid Qwen response format: missing content".into()))?;

    Ok(text.to_string())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/seller")
        .add("/verify", post(verify_seller))
}
