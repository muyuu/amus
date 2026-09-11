//! 書き起こしスレッドとのやり取り（リクエスト送信・結果ポーリング）

use super::{VoiceMemo, VoiceMemoFeature};
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::TranscribeRequest;
use crate::{log_debug, log_error};

impl VoiceMemoFeature {
    /// 書き起こし結果をポーリング
    pub(super) fn poll_transcription_results(&mut self) {
        if let Some(transcriber) = &self.transcriber_thread {
            let results = transcriber.poll_results();
            for result in results {
                self.state.pending_chunks = self.state.pending_chunks.saturating_sub(1);

                if let Some(error) = result.error {
                    log_error!("VoiceMemo", &format!("書き起こしエラー: {}", error));
                    self.state.error = Some(error);
                    continue;
                }

                // 結果をラウンドに追加
                if let Some(round) = self.state.rounds.get_mut(result.round_index) {
                    let mut added_count = 0;
                    for seg in result.segments {
                        if !seg.text.trim().is_empty() {
                            round.memos.push(VoiceMemo {
                                timestamp_secs: seg.timestamp_secs,
                                text: seg.text.trim().to_string(),
                            });
                            added_count += 1;
                        }
                    }
                    // タイムスタンプでソート
                    round.memos.sort_by(|a, b| {
                        a.timestamp_secs
                            .partial_cmp(&b.timestamp_secs)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    log_debug!(
                        "VoiceMemo",
                        &format!("解析完了: {}件のメモを追加", added_count)
                    );
                }
            }

            // 処理待ちがなくなったらis_processingをfalseに
            if self.state.pending_chunks == 0 {
                self.state.is_processing = false;
            }
        }
    }

    /// 書き起こしリクエストを送信
    /// `start_sample` は発話区間の先頭の絶対インデックス（`SAMPLE_RATE` 基準）。
    ///
    /// 進行中のターンがない場合は送らない。メモはターンに属するため。
    pub(super) fn send_transcription_request(&mut self, samples: Vec<f32>, start_sample: usize) {
        if !self.state.round_active {
            return;
        }

        if let Some(transcriber) = &self.transcriber_thread {
            let current_round = self.state.rounds.len().saturating_sub(1);
            let round_start = self
                .state
                .rounds
                .last()
                .map(|round| round.start_sample)
                .unwrap_or(0);

            // メモのタイムスタンプはターン開始からの相対秒
            let chunk_start_secs =
                start_sample.saturating_sub(round_start) as f32 / SAMPLE_RATE as f32;

            let request = TranscribeRequest {
                samples,
                offset_secs: chunk_start_secs,
                context: self.context.clone(),
                round_index: current_round,
            };

            transcriber.request(request);
            self.state.pending_chunks += 1;
            self.state.is_processing = true;

            log_debug!(
                "VoiceMemo",
                &format!(
                    "発話チャンク送信: offset={:.1}s, round={}",
                    chunk_start_secs, current_round
                )
            );
        }
    }
}
