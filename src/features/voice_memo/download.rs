//! Whisper モデルのダウンロード制御

use super::VoiceMemoFeature;
use crate::resources::{
    download_file, get_model_download_url, get_model_path, Resources, TranscriberThread,
};
use crate::{log_debug, log_error};

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

        let url = get_model_download_url().to_string();
        let dest_path = get_model_path();

        let handle = std::thread::spawn(move || download_file(&url, &dest_path, tx));
        self.download_handle = Some(handle);

        // ダウンロード完了後に Resources の whisper_transcriber を初期化する必要がある
        // これは update_download_progress で処理される
        let _ = resources; // 将来の拡張用
    }

    pub(super) fn update_download_progress(&mut self, _resources: &Resources) {
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
                            // TranscriberThread を初期化
                            if self.transcriber_thread.is_none() {
                                match TranscriberThread::new() {
                                    Ok(t) => {
                                        self.transcriber_thread = Some(t);
                                        log_debug!("VoiceMemo", "TranscriberThread初期化完了");
                                    }
                                    Err(e) => {
                                        log_error!(
                                            "VoiceMemo",
                                            &format!("TranscriberThread初期化エラー: {}", e)
                                        );
                                    }
                                }
                            }
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
