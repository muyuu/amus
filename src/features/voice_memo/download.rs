//! Whisper モデルのダウンロード制御

use super::VoiceMemoFeature;
use crate::log_debug;
use crate::resources::{download_file, IntegrityCheck, Resources};

impl VoiceMemoFeature {
    pub(super) fn start_download(&mut self, resources: &mut Resources) {
        if self.state.is_downloading {
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.download_rx = Some(rx);
        self.state.is_downloading = true;
        self.state.download_progress = Some(0.0);
        self.state.error = None;

        let model = resources.transcribe_setup().model;
        let url = model.url().to_string();
        let dest_path = model.path();

        // 前回終了時に立てたフラグが残っていることはないが、開始時に必ず倒す
        self.download_cancel
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let cancel = std::sync::Arc::clone(&self.download_cancel);

        let handle = std::thread::spawn(move || {
            let check = IntegrityCheck {
                max_bytes: Some(model.max_download_bytes()),
                sha256_hex: Some(model.sha256()),
            };
            download_file(&url, &dest_path, tx, cancel, &check)
        });
        self.download_handle = Some(handle);
    }

    pub(super) fn update_download_progress(&mut self, resources: &mut Resources) {
        if let Some(rx) = &self.download_rx {
            let mut latest_progress = None;
            while let Ok(progress) = rx.try_recv() {
                latest_progress = Some(progress);
            }

            if let Some(progress) = latest_progress {
                self.state.download_progress = progress.percentage();
                self.state.downloaded_bytes = progress.downloaded_bytes;
                self.state.total_bytes = progress.total_bytes;
            }
        }

        if self.state.is_downloading {
            if let Some(handle) = self.download_handle.take() {
                if handle.is_finished() {
                    match handle.join() {
                        Ok(Ok(())) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.model_available = true;
                            self.state.error = None;

                            // 取得したモデルで書き起こしを用意する。認識コンテキストは
                            // モデルのトークナイザで組むため、組み直させる。
                            resources.reload_transcriber();
                            self.sync_backend_state(resources);
                            self.context_vocabulary = None;
                            self.restart_transcriber_thread(resources);
                            log_debug!("VoiceMemo", "書き起こしの初期化完了");
                        }
                        Ok(Err(e)) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.error = Some(e.to_string());
                        }
                        Err(_) => {
                            self.state.is_downloading = false;
                            self.state.download_progress = None;
                            self.download_rx = None;
                            self.state.error = Some("ダウンロードスレッドがパニック".to_string());
                        }
                    }
                } else {
                    self.download_handle = Some(handle);
                }
            }
        }
    }
}
