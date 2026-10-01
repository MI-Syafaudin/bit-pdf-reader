use crate::config::OcrConfig;
use crate::pipeline::{OcrPipeline, OcrProgress, OcrResult};
use crossbeam_channel::{bounded, Receiver, Sender};
use pdfx_core::cancellation::CancellationToken;
use pdfx_core::errors::PdfxError;
use std::path::PathBuf;
use std::thread;

pub type OcrTaskResult = Result<OcrResult, PdfxError>;

pub struct OcrJobHandle {
    pub progress_rx: Receiver<OcrProgress>,
    pub result_rx: Receiver<OcrTaskResult>,
    pub cancel_token: CancellationToken,
}

impl OcrJobHandle {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}

pub struct OcrWorker;

impl OcrWorker {
    pub fn spawn_job(
        doc_path: PathBuf,
        pages: Vec<usize>,
        config: OcrConfig,
        output_path: Option<PathBuf>,
    ) -> OcrJobHandle {
        let (progress_tx, progress_rx): (Sender<OcrProgress>, Receiver<OcrProgress>) = bounded(50);
        let (result_tx, result_rx): (Sender<OcrTaskResult>, Receiver<OcrTaskResult>) = bounded(1);
        let cancel_token = CancellationToken::new();

        let cancel_clone = cancel_token.clone();
        thread::spawn(move || {
            let cb = move |progress: OcrProgress| {
                let _ = progress_tx.send(progress);
            };

            let res = OcrPipeline::process_document(
                &doc_path,
                &pages,
                &config,
                output_path.as_deref(),
                &cancel_clone,
                Some(&cb),
            );

            let _ = result_tx.send(res);
        });

        OcrJobHandle {
            progress_rx,
            result_rx,
            cancel_token,
        }
    }
}
