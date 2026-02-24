use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::io::{AsyncWriteExt, BufWriter};
use uuid::Uuid;

use crate::hls;
use crate::storage::StorageBackend;
use crate::validation;
use tracing::info;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

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

pub type SharedStorage = Arc<dyn StorageBackend>;

pub async fn upload_audio(
    State(storage): State<SharedStorage>,
    mut multipart: Multipart,
) -> (StatusCode, Json<UploadResponse>) {
    let mut field = match multipart.next_field().await {
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

    let suffix = format!(".{}", extension);
    let named_tmp = match NamedTempFile::with_suffix(&suffix) {
        Ok(f) => f,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Не удалось создать временный файл: {}", e),
            );
        }
    };

    let tmp_path = named_tmp.into_temp_path();

    let file = match tokio::fs::File::create(&tmp_path).await {
        Ok(f) => f,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Не удалось открыть временный файл для записи: {}", e),
            );
        }
    };
    let mut writer = BufWriter::new(file);

    let mut total_bytes: usize = 0;
    let mut magic_checked = false;
    let mut head_buf = Vec::new();

    info!("Start receiving file '{}' (ext={})", filename, extension);

    loop {
        let chunk = match field.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => break,
            Err(e) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    format!("Ошибка чтения данных файла: {}", e),
                );
            }
        };

        total_bytes += chunk.len();

        if total_bytes > MAX_FILE_SIZE {
            return error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "Файл слишком большой: {} MB (максимум: {} MB)",
                    total_bytes / (1024 * 1024),
                    MAX_FILE_SIZE / (1024 * 1024)
                ),
            );
        }

        if !magic_checked {
            head_buf.extend_from_slice(&chunk);
            if head_buf.len() >= 12 {
                let detected_format = match validation::validate_magic_bytes(&head_buf) {
                    Ok(fmt) => fmt,
                    Err(e) => {
                        return error_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, e);
                    }
                };
                if let Err(e) = validation::check_extension_magic_compatibility(
                    &extension,
                    detected_format,
                ) {
                    return error_response(StatusCode::BAD_REQUEST, e);
                }
                info!(
                    "File '{}' passed magic bytes check: ext={}, magic={}",
                    filename, extension, detected_format
                );
                magic_checked = true;
                head_buf = Vec::new();
            }
        }

        if let Err(e) = writer.write_all(&chunk).await {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Ошибка записи во временный файл: {}", e),
            );
        }
    }

    if let Err(e) = writer.flush().await {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка сброса буфера во временный файл: {}", e),
        );
    }

    if total_bytes == 0 {
        return error_response(StatusCode::BAD_REQUEST, "Файл пустой".into());
    }

    if !magic_checked {
        return error_response(
            StatusCode::BAD_REQUEST,
            "Файл слишком мал для определения формата".into(),
        );
    }

    info!(
        "File '{}' received: {} bytes, saved to {}",
        filename,
        total_bytes,
        tmp_path.display()
    );

    let tmp_pathbuf = tmp_path.to_path_buf();

    let validation_result = match run_ffprobe_validation(&tmp_pathbuf).await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if let Err(resp) = run_decode_check(&tmp_pathbuf, &filename).await {
        return resp;
    }

    let hls_output = match run_hls_conversion(&tmp_pathbuf).await {
        Ok(h) => h,
        Err(resp) => return resp,
    };

    let upload_result = upload_hls_to_storage(&storage, &hls_output, &filename).await;

    hls_output.cleanup().await;

    let (hls_path, _) = match upload_result {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    (
        StatusCode::OK,
        Json(UploadResponse {
            success: true,
            message: Some(
                "Аудиофайл прошёл валидацию, конвертирован в HLS и загружен в хранилище".into(),
            ),
            filename: Some(filename.to_string()),
            format: validation_result.format_name,
            codec: validation_result.codec,
            sample_rate: validation_result.sample_rate,
            channels: validation_result.channels,
            duration_secs: validation_result.duration_secs,
            bit_rate: validation_result.bit_rate,
            size_bytes: Some(total_bytes),
            hls_path: Some(hls_path),
            error: None,
        }),
    )
}

type ErrorResponse = (StatusCode, Json<UploadResponse>);

async fn run_ffprobe_validation(
    tmp_path: &PathBuf,
) -> Result<validation::ValidationResult, ErrorResponse> {
    let probe_path = tmp_path.clone();
    let probe_result =
        tokio::task::spawn_blocking(move || validation::run_ffprobe(&probe_path)).await;

    let validation_result = match probe_result {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => return Err(error_response(StatusCode::UNPROCESSABLE_ENTITY, e)),
        Err(e) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ffprobe task failed: {}", e),
            ))
        }
    };

    info!(
        "ffprobe OK: codec={:?}, sr={:?}, ch={:?}, dur={:?}",
        validation_result.codec,
        validation_result.sample_rate,
        validation_result.channels,
        validation_result.duration_secs
    );

    Ok(validation_result)
}

async fn run_decode_check(tmp_path: &PathBuf, filename: &str) -> Result<(), ErrorResponse> {
    let decode_path = tmp_path.clone();
    let decode_result =
        tokio::task::spawn_blocking(move || validation::run_ffmpeg_decode_check(&decode_path))
            .await;

    match decode_result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(error_response(StatusCode::UNPROCESSABLE_ENTITY, e)),
        Err(e) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("ffmpeg decode task failed: {}", e),
            ))
        }
    }

    info!("ffmpeg decode check passed for '{}'", filename);
    Ok(())
}

async fn run_hls_conversion(tmp_path: &PathBuf) -> Result<hls::HlsOutput, ErrorResponse> {
    let hls_input = tmp_path.clone();
    let hls_result = tokio::task::spawn_blocking(move || hls::convert_to_hls(&hls_input)).await;

    let hls_output = match hls_result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HLS конвертация не удалась: {}", e),
            ))
        }
        Err(e) => {
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("HLS task failed: {}", e),
            ))
        }
    };

    info!("HLS conversion OK: dir={}", hls_output.output_dir.display());
    Ok(hls_output)
}

async fn upload_hls_to_storage(
    storage: &SharedStorage,
    hls_output: &hls::HlsOutput,
    filename: &str,
) -> Result<(String, String), ErrorResponse> {
    let file_stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("audio");
    let upload_prefix = format!("{}/{}", file_stem, Uuid::new_v4());

    if let Err(e) = storage.ensure_bucket(HLS_BUCKET).await {
        return Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Не удалось создать бакет: {}", e),
        ));
    }

    if let Err(e) = storage
        .upload_hls_output(hls_output, HLS_BUCKET, &upload_prefix)
        .await
    {
        return Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Ошибка загрузки в хранилище: {}", e),
        ));
    }

    let hls_path = format!("{}/{}/playlist.m3u8", HLS_BUCKET, upload_prefix);
    info!("Upload complete: {}", hls_path);

    Ok((hls_path, upload_prefix))
}
