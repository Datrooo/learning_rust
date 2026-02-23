use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use uuid::Uuid;

use crate::hls;
use crate::storage::StorageBackend;
use crate::validation;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

/// Имя бакета для HLS аудио
const HLS_BUCKET: &str = "audio-hls";

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hls_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn error_response(status: StatusCode, error: String) -> (StatusCode, Json<UploadResponse>) {
    tracing::warn!("Validation failed: {}", error);
    (
        status,
        Json(UploadResponse {
            success: false,
            message: None,
            filename: None,
            format: None,
            codec: None,
            sample_rate: None,
            channels: None,
            duration_secs: None,
            bit_rate: None,
            size_bytes: None,
            hls_path: None,
            error: Some(error),
        }),
    )
}

/// Shared state — хранилище, доступное всем хэндлерам
pub type SharedStorage = Arc<dyn StorageBackend>;

pub async fn upload_audio(
    State(storage): State<SharedStorage>,
    mut multipart: Multipart,
) -> (StatusCode, Json<UploadResponse>) {
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            return error_response(StatusCode::BAD_REQUEST, "Файл не найден в запросе".into());
        }
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!("Ошибка чтения multipart: {}", e),
            );
        }
    };

    let filename = field.file_name().unwrap_or("unknown").to_string();

    let extension = match validation::validate_extension(&filename) {
        Ok(ext) => ext,
        Err(e) => {
            return error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
        }
    };

    // мб переделать через чанки
    let data = match field.bytes().await {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                format!("Ошибка чтения данных файла: {}", e),
            );
        }
    };

    if data.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "Файл пустой".into());
    }

    let size_bytes = data.len();

    if size_bytes > MAX_FILE_SIZE {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "Файл слишком большой: {} MB (максимум: {} MB)",
                size_bytes / (1024 * 1024),
                MAX_FILE_SIZE / (1024 * 1024)
            ),
        );
    }

    let detected_format = match validation::validate_magic_bytes(&data) {
        Ok(fmt) => fmt,
        Err(e) => {
            return error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
        }
    };

    if let Err(e) = validation::check_extension_magic_compatibility(&extension, detected_format) {
        return error_response(StatusCode::BAD_REQUEST, e);
    }

    tracing::info!(
        "File '{}' passed quick checks: ext={}, magic={}",
        filename,
        extension,
        detected_format
    );

    let tmp_dir = std::env::temp_dir();
    let tmp_filename = format!("{}.{}", Uuid::new_v4(), extension);
    let tmp_path: PathBuf = tmp_dir.join(&tmp_filename);

    if let Err(e) = fs::write(&tmp_path, &data).await {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Не удалось сохранить временный файл: {}", e),
        );
    }

    let result = run_deep_validation(&tmp_path, &filename, &extension, size_bytes, &storage).await;

    // добавить raii
    let _ = fs::remove_file(&tmp_path).await;

    result
}

async fn run_deep_validation(
    tmp_path: &PathBuf,
    filename: &str,
    _extension: &str,
    size_bytes: usize,
    storage: &SharedStorage,
) -> (StatusCode, Json<UploadResponse>) {
    let probe_path = tmp_path.clone();
    let probe_result =
        tokio::task::spawn_blocking(move || validation::run_ffprobe(&probe_path)).await;

    let validation_result = match probe_result {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            return error_response(StatusCode::UNPROCESSABLE_ENTITY, e);
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ffprobe task failed: {}", e),
            );
        }
    };

    tracing::info!(
        "ffprobe OK: codec={:?}, sr={:?}, ch={:?}, dur={:?}",
        validation_result.codec,
        validation_result.sample_rate,
        validation_result.channels,
        validation_result.duration_secs
    );

    let decode_path = tmp_path.clone();
    let decode_result =
        tokio::task::spawn_blocking(move || validation::run_ffmpeg_decode_check(&decode_path))
            .await;

    match decode_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            return error_response(StatusCode::UNPROCESSABLE_ENTITY, e);
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ffmpeg decode task failed: {}", e),
            );
        }
    }

    tracing::info!("ffmpeg decode check passed for '{}'", filename);

    // ─── Шаг 6: HLS конвертация ─────────────────────────────────────────
    let hls_input = tmp_path.clone();
    let hls_result =
        tokio::task::spawn_blocking(move || hls::convert_to_hls(&hls_input)).await;

    let hls_output = match hls_result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HLS конвертация не удалась: {}", e),
            );
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HLS task failed: {}", e),
            );
        }
    };

    tracing::info!(
        "HLS conversion OK: dir={}",
        hls_output.output_dir.display()
    );

    // ─── Шаг 7: Upload в S3 хранилище ──────────────────────────────────
    // Prefix: "имя_файла_без_расширения/uuid" для уникальности
    let file_stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let upload_prefix = format!("{}/{}", file_stem, Uuid::new_v4());

    // Создаём бакет если не существует
    if let Err(e) = storage.ensure_bucket(HLS_BUCKET).await {
        hls_output.cleanup().await;
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Не удалось создать бакет: {}", e),
        );
    }

    // Загружаем все HLS файлы
    if let Err(e) = storage
        .upload_hls_output(&hls_output, HLS_BUCKET, &upload_prefix)
        .await
    {
        hls_output.cleanup().await;
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка загрузки в хранилище: {}", e),
        );
    }

    // Cleanup tmp HLS файлов
    hls_output.cleanup().await;

    let hls_path = format!("{}/{}/playlist.m3u8", HLS_BUCKET, upload_prefix);
    tracing::info!("Upload complete: {}", hls_path);

    (
        StatusCode::OK,
        Json(UploadResponse {
            success: true,
            message: Some("Аудиофайл прошёл валидацию, конвертирован в HLS и загружен в хранилище".into()),
            filename: Some(filename.to_string()),
            format: validation_result.format_name,
            codec: validation_result.codec,
            sample_rate: validation_result.sample_rate,
            channels: validation_result.channels,
            duration_secs: validation_result.duration_secs,
            bit_rate: validation_result.bit_rate,
            size_bytes: Some(size_bytes),
            hls_path: Some(hls_path),
            error: None,
        }),
    )
}
