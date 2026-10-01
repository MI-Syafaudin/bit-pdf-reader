use crate::config::OcrConfig;
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info};

pub struct TesseractRunner {
    executable_path: String,
}

impl TesseractRunner {
    pub fn detect() -> Result<Self, PdfxError> {
        let output = Command::new("tesseract").arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                let version_str = String::from_utf8_lossy(&out.stdout);
                let first_line = version_str.lines().next().unwrap_or("tesseract");
                info!("Detected OCR engine: {}", first_line);
                Ok(Self {
                    executable_path: "tesseract".to_string(),
                })
            }
            _ => Err(PdfxError::Ocr(
                "Tesseract OCR is not installed or not in PATH. Please install tesseract-ocr to enable OCR features (e.g. 'sudo pacman -S tesseract' or 'sudo apt install tesseract-ocr').".to_string(),
            )),
        }
    }

    pub fn available_languages(&self) -> Result<Vec<String>, PdfxError> {
        let output = Command::new(&self.executable_path)
            .arg("--list-langs")
            .output()
            .map_err(|e| PdfxError::Ocr(format!("Failed to query languages: {}", e)))?;

        if !output.status.success() {
            return Err(PdfxError::Ocr(
                "Failed to list tesseract languages".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let langs: Vec<String> = stdout
            .lines()
            .skip(1) // Skip "List of available languages"
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        debug!("Available OCR languages: {:?}", langs);
        Ok(langs)
    }

    pub fn has_language(&self, lang: &str) -> Result<bool, PdfxError> {
        let langs = self.available_languages()?;
        for l in lang.split('+') {
            if !langs.iter().any(|item| item == l) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn run_ocr_on_image(
        &self,
        image_path: &Path,
        output_base: &Path,
        lang: &str,
        cancel_token: &CancellationToken,
    ) -> Result<(), PdfxError> {
        let config = OcrConfig {
            language: lang.to_string(),
            ..Default::default()
        };
        self.run_ocr(image_path, output_base, &config, cancel_token)?;
        Ok(())
    }

    pub fn run_ocr(
        &self,
        image_path: &Path,
        output_base: &Path,
        config: &OcrConfig,
        cancel_token: &CancellationToken,
    ) -> Result<PathBuf, PdfxError> {
        if cancel_token.is_cancelled() {
            return Err(PdfxError::Cancelled);
        }

        let mut cmd = Command::new(&self.executable_path);
        cmd.arg(image_path);
        cmd.arg(output_base);
        cmd.args(["-l", &config.language]);

        if let Some(psm) = config.psm {
            cmd.args(["--psm", psm.psm_number()]);
        }

        cmd.arg(config.format.tesseract_config_flag());

        debug!("Executing OCR command: {:?}", cmd);

        let mut child = cmd
            .spawn()
            .map_err(|e| PdfxError::Ocr(format!("Failed to spawn Tesseract process: {}", e)))?;

        // Poll child with cancellation check
        loop {
            if cancel_token.is_cancelled() {
                let _ = child.kill();
                return Err(PdfxError::Cancelled);
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        return Err(PdfxError::Ocr(format!(
                            "Tesseract exited with non-zero code: {:?}",
                            status.code()
                        )));
                    }
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(PdfxError::Ocr(format!(
                        "Error waiting for Tesseract: {}",
                        e
                    )));
                }
            }
        }

        let ext = config.format.file_extension();
        let final_output = output_base.with_extension(ext);
        Ok(final_output)
    }
}
