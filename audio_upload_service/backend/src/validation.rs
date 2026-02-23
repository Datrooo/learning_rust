use serde::Deserialize;
use std::path::Path;
use std::process::Command;

const ALLOWED_EXTENSIONS: &[&str] = &["mp3", "wav", "ogg", "flac", "opus", "m4a", "aac"];
const MAX_DURATION_SECS: f64 = 3600.0;
const MIN_SAMPLE_RATE: u32 = 8000;
const MAX_SAMPLE_RATE: u32 = 192000;
const MAX_CHANNELS: u32 = 8;

#[derive(Debug, Deserialize)]
pub struct FfprobeOutput {
    pub streams: Option<Vec<FfprobeStream>>,
    pub format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
pub struct FfprobeStream {
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub duration: Option<String>,
    pub bit_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FfprobeFormat {
    pub format_name: Option<String>,
    pub duration: Option<String>,
    pub bit_rate: Option<String>,
    pub nb_streams: Option<u32>,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub format_name: Option<String>,
    pub codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub duration_secs: Option<f64>,
    pub bit_rate: Option<u64>,
}

// добавить валидацию в случае, если точки нет вообще, или она в начале (скрытый файл), или в конце (пустое расширение, вроде учитываю)
pub fn validate_extension(filename: &str) -> Result<String, String> {
    let ext = filename
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .ok_or_else(|| "Файл не имеет расширения".to_string())?;

    if ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        Ok(ext)
    } else {
        Err(format!(
            "Недопустимое расширение '.{}'. Разрешены: {}",
            ext,
            ALLOWED_EXTENSIONS.join(", ")
        ))
    }
}

pub fn validate_magic_bytes(data: &[u8]) -> Result<&'static str, String> {
    if data.len() < 12 {
        return Err("Файл слишком мал для определения формата".to_string());
    }

    if &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return Ok("wav");
    }

    if &data[0..4] == b"fLaC" {
        return Ok("flac");
    }

    if &data[0..4] == b"OggS" {
        return Ok("ogg");
    }

    if data.len() >= 3 && &data[0..3] == b"ID3" {
        return Ok("mp3");
    }

    if data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        let layer_bits = data[1] & 0x06;
        if layer_bits == 0x00 && (data[1] & 0xF0) == 0xF0 {
            return Ok("aac");
        } else {
            return Ok("mp3");
        }
    }

    if data.len() >= 8 && &data[4..8] == b"ftyp" {
        return Ok("m4a");
    }

    Err("Не удалось определить формат по заголовку файла. Файл не является аудио.".to_string())
}

pub fn check_extension_magic_compatibility(extension: &str, detected: &str) -> Result<(), String> {
    let compatible = match (extension, detected) {
        (a, b) if a == b => true,
        ("opus", "ogg") => true,
        _ => false,
    };

    if !compatible {
        return Err(format!(
            "Расширение '.{}' не соответствует реальному формату '{}'. Файл переименован?",
            extension, detected
        ));
    }

    Ok(())
}

pub fn run_ffprobe(file_path: &Path) -> Result<ValidationResult, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            "-select_streams",
            "a:0",
        ])
        .arg(file_path)
        .output()
        .map_err(|e| format!("Не удалось запустить ffprobe: {}. Установлен ли ffmpeg?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ffprobe не смог прочитать файл: {}",
            if stderr.trim().is_empty() {
                "неизвестная ошибка".to_string()
            } else {
                stderr.trim().to_string()
            }
        ));
    }

    let probe: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Не удалось распарсить вывод ffprobe: {}", e))?;

    let streams = probe
        .streams
        .as_ref()
        .ok_or_else(|| "ffprobe не обнаружил потоков в файле".to_string())?;

    let audio_stream = streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("audio"))
        .ok_or_else(|| "Файл не содержит аудиопотока".to_string())?;

    let codec = audio_stream.codec_name.clone();

    let sample_rate: Option<u32> = audio_stream
        .sample_rate
        .as_ref()
        .and_then(|s| s.parse().ok());

    let channels = audio_stream.channels;

    let duration_secs: Option<f64> = audio_stream
        .duration
        .as_ref()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            probe
                .format
                .as_ref()
                .and_then(|f| f.duration.as_ref())
                .and_then(|s| s.parse().ok())
        });

    let bit_rate: Option<u64> = audio_stream
        .bit_rate
        .as_ref()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            probe
                .format
                .as_ref()
                .and_then(|f| f.bit_rate.as_ref())
                .and_then(|s| s.parse().ok())
        });

    let format_name = probe.format.and_then(|f| f.format_name);

    if let Some(dur) = duration_secs {
        if dur <= 0.0 {
            return Err("Аудиофайл имеет нулевую или отрицательную длительность".to_string());
        }
        if dur > MAX_DURATION_SECS {
            return Err(format!(
                "Аудиофайл слишком длинный: {:.0} сек (максимум: {:.0} сек)",
                dur, MAX_DURATION_SECS
            ));
        }
    }

    if let Some(sr) = sample_rate {
        if sr < MIN_SAMPLE_RATE || sr > MAX_SAMPLE_RATE {
            return Err(format!(
                "Недопустимый sample rate: {} Hz (допустимо: {}–{} Hz)",
                sr, MIN_SAMPLE_RATE, MAX_SAMPLE_RATE
            ));
        }
    }

    if let Some(ch) = channels {
        if ch == 0 || ch > MAX_CHANNELS {
            return Err(format!(
                "Недопустимое количество каналов: {} (допустимо: 1–{})",
                ch, MAX_CHANNELS
            ));
        }
    }

    Ok(ValidationResult {
        format_name,
        codec,
        sample_rate,
        channels,
        duration_secs,
        bit_rate,
    })
}

pub fn run_ffmpeg_decode_check(file_path: &Path) -> Result<(), String> {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(file_path)
        .args(["-f", "null", "-"])
        .output()
        .map_err(|e| format!("Не удалось запустить ffmpeg: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr_trimmed = stderr.trim();

    if !output.status.success() || !stderr_trimmed.is_empty() {
        let error_msg = if stderr_trimmed.is_empty() {
            format!("ffmpeg завершился с кодом {}", output.status)
        } else {
            let truncated: String = stderr_trimmed.chars().take(500).collect();
            format!("Ошибки декодирования: {}", truncated)
        };
        return Err(error_msg);
    }

    Ok(())
}
