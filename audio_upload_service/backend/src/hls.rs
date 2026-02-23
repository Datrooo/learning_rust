use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug)]
pub struct HlsOutput {
    pub output_dir: PathBuf,
    pub playlist_name: String,
}

impl HlsOutput {
    pub fn playlist_path(&self) -> PathBuf {
        self.output_dir.join(&self.playlist_name)
    }

    pub async fn list_files(&self) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        let mut entries = fs::read_dir(&self.output_dir)
            .await
            .map_err(|e| format!("Не удалось прочитать HLS директорию: {}", e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Ошибка чтения HLS файла: {}", e))?
        {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            }
        }

        files.sort();
        Ok(files)
    }

    pub async fn cleanup(&self) {
        let _ = fs::remove_dir_all(&self.output_dir).await;
    }
}

pub fn convert_to_hls(input_path: &Path) -> Result<HlsOutput, String> {
    let hls_dir = std::env::temp_dir().join(format!("hls_{}", Uuid::new_v4()));

    std::fs::create_dir_all(&hls_dir)
        .map_err(|e| format!("Не удалось создать HLS директорию: {}", e))?;

    let playlist_path = hls_dir.join("playlist.m3u8");
    let segment_pattern = hls_dir.join("seg_%05d.m4s");

    let output = Command::new("ffmpeg")
        .args([
            "-i",
            input_path.to_str().unwrap_or(""),
            "-v",
            "error",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-ac",
            "2",
            "-ar",
            "48000",
            "-f",
            "hls",
            "-hls_time",
            "6",
            "-hls_playlist_type",
            "vod",
            "-hls_segment_type",
            "fmp4",
            "-hls_segment_filename",
            segment_pattern.to_str().unwrap_or(""),
        ])
        .arg(playlist_path.to_str().unwrap_or(""))
        .output()
        .map_err(|e| format!("Не удалось запустить ffmpeg для HLS: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_dir_all(&hls_dir);
        return Err(format!(
            "ffmpeg HLS конвертация не удалась: {}",
            if stderr.trim().is_empty() {
                format!("exit code {}", output.status)
            } else {
                stderr.trim().to_string()
            }
        ));
    }

    if !playlist_path.exists() {
        let _ = std::fs::remove_dir_all(&hls_dir);
        return Err("ffmpeg не создал HLS плейлист".to_string());
    }

    Ok(HlsOutput {
        output_dir: hls_dir,
        playlist_name: "playlist.m3u8".to_string(),
    })
}
