//! 書き起こしスレッドとのやり取り（リクエスト送信・結果ポーリング）

use super::hotword::{Hotword, TurnBoundary};
use super::{VoiceMemo, VoiceMemoFeature};
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::TranscribeRequest;
use crate::{log_debug, log_error};

impl VoiceMemoFeature {
    /// 書き起こし結果をポーリングして取り込む。
    ///
    /// セグメントは録音上の順に処理する。トリガーワードで開いたターンへ、同じチャンクの
    /// 後続セグメントがそのまま入るようにするため。
    pub(super) fn poll_transcription_results(&mut self) {
        let Some(transcriber) = &self.transcriber_thread else {
            return;
        };

        for result in transcriber.poll_results() {
            self.state.pending_chunks = self.state.pending_chunks.saturating_sub(1);

            if let Some(error) = result.error {
                log_error!("VoiceMemo", &format!("書き起こしエラー: {}", error));
                self.state.error = Some(error);
                continue;
            }

            for segment in result.segments {
                let text = segment.text.trim();
                if text.is_empty() {
                    continue;
                }
                self.apply_segment(segment.start_sample, segment.end_sample, text);
            }
        }

        // 処理待ちがなくなったらis_processingをfalseに
        if self.state.pending_chunks == 0 {
            self.state.is_processing = false;
        }
    }

    /// 発話チャンクを書き起こしへ送る。
    /// `start_sample` は発話区間の先頭の絶対インデックス（`SAMPLE_RATE` 基準）。
    ///
    /// ターンの内外を問わず送る。トリガーワードは書き起こし結果から拾うため、ターンが
    /// 開く前の発話も書き起こす必要がある。ターンへの所属は結果を受け取った時点で決まる。
    pub(super) fn send_transcription_request(&mut self, samples: Vec<f32>, start_sample: usize) {
        let Some(transcriber) = &self.transcriber_thread else {
            return;
        };

        transcriber.request(TranscribeRequest {
            samples,
            start_sample,
            context: self.context.clone(),
        });
        self.state.pending_chunks += 1;
        self.state.is_processing = true;

        log_debug!(
            "VoiceMemo",
            &format!("発話チャンク送信: start_sample={}", start_sample)
        );
    }

    /// 指定位置の発話をメモとして積む。
    ///
    /// `at` は録音上の絶対サンプル位置。その位置を含むターンへ入れ、どのターンにも
    /// 属さなければ捨てる。書き起こしはターンの開閉より遅れて返るため、所属は
    /// 送信時点ではなく位置で決める。
    pub(super) fn push_memo_at(&mut self, at: usize, text: String) {
        let Some(index) = self.round_index_at(at) else {
            return;
        };

        let round = &mut self.state.rounds[index];
        let timestamp_secs = at.saturating_sub(round.start_sample) as f32 / SAMPLE_RATE as f32;
        round.memos.push(VoiceMemo {
            timestamp_secs,
            text,
        });
        round.memos.sort_by(|a, b| {
            a.timestamp_secs
                .partial_cmp(&b.timestamp_secs)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// 指定位置を含むターンの位置。進行中のターンは終端を持たないため末尾まで含む。
    fn round_index_at(&self, at: usize) -> Option<usize> {
        self.state.rounds.iter().position(|round| {
            at >= round.start_sample && round.end_sample.is_none_or(|end| at < end)
        })
    }

    /// 書き起こしセグメントを1件取り込む。位置は録音上の絶対サンプル位置。
    ///
    /// トリガーワードを含むセグメントはターン境界の指示として扱い、メモにはしない。
    /// 境界はセグメントの終端に置くため、ワード自体の発話はターンに入らない。
    ///
    /// 録音を取り直した後に前の録音の結果が返ることがある。位置が現在の録音より前の
    /// セグメントは捨てる。
    pub(super) fn apply_segment(&mut self, start_sample: usize, end_sample: usize, text: &str) {
        if start_sample < self.recording_start_sample {
            return;
        }

        if let Some(hotword) = self.matcher.find(text) {
            self.move_turn_boundary(hotword, end_sample);
            return;
        }

        self.push_memo_at(start_sample, text.to_string());
    }

    /// 検出したトリガーワードに応じてターン境界を動かす。
    fn move_turn_boundary(&mut self, hotword: Hotword, at: usize) {
        let Some(boundary) = self
            .boundary_tracker
            .accept(hotword, at, self.state.round_active)
        else {
            return;
        };

        log_debug!(
            "VoiceMemo",
            &format!(
                "ホットワード検知: {:?} -> {:?} at {}",
                hotword, boundary, at
            )
        );

        match boundary {
            TurnBoundary::Open => self.open_turn(at),
            TurnBoundary::Close => self.close_turn(at),
            TurnBoundary::Restart => {
                self.close_turn(at);
                self.open_turn(at);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::voice_memo::Round;
    use crate::resources::voice_recorder::SAMPLE_RATE;

    /// `[start, end)` のターンを並べた feature を作る。`None` は進行中のターン。
    fn feature_with_turns(turns: &[(usize, Option<usize>)]) -> VoiceMemoFeature {
        let mut feature = VoiceMemoFeature::default();
        feature.state.rounds = turns
            .iter()
            .map(|&(start_sample, end_sample)| Round {
                start_sample,
                end_sample,
                ..Default::default()
            })
            .collect();
        feature
    }

    fn memo_texts(feature: &VoiceMemoFeature, round: usize) -> Vec<&str> {
        feature.state.rounds[round]
            .memos
            .iter()
            .map(|memo| memo.text.as_str())
            .collect()
    }

    #[test]
    fn a_memo_goes_to_the_turn_containing_its_position() {
        let mut feature = feature_with_turns(&[(0, Some(100)), (200, Some(300))]);

        feature.push_memo_at(50, "1つ目のターン".to_string());
        feature.push_memo_at(250, "2つ目のターン".to_string());

        assert_eq!(memo_texts(&feature, 0), ["1つ目のターン"]);
        assert_eq!(memo_texts(&feature, 1), ["2つ目のターン"]);
    }

    #[test]
    fn a_memo_outside_every_turn_is_dropped() {
        let mut feature = feature_with_turns(&[(0, Some(100)), (200, Some(300))]);

        // ターンの切れ目の発話はどのターンにも属さない
        feature.push_memo_at(150, "ターン外".to_string());

        assert!(memo_texts(&feature, 0).is_empty());
        assert!(memo_texts(&feature, 1).is_empty());
    }

    #[test]
    fn an_open_turn_takes_memos_after_its_start() {
        let mut feature = feature_with_turns(&[(100, None)]);

        feature.push_memo_at(50, "開始前".to_string());
        feature.push_memo_at(100_000, "開始後".to_string());

        assert_eq!(memo_texts(&feature, 0), ["開始後"]);
    }

    #[test]
    fn the_timestamp_is_measured_from_the_start_of_its_turn() {
        let mut feature = feature_with_turns(&[(SAMPLE_RATE as usize, None)]);

        feature.push_memo_at(SAMPLE_RATE as usize * 4, "3秒地点".to_string());

        assert_eq!(feature.state.rounds[0].memos[0].timestamp_secs, 3.0);
    }
}

#[cfg(test)]
mod hotword_tests {
    use super::*;
    use crate::features::voice_memo::hotword::BoundaryTracker;

    /// クールダウンは検証したい振る舞いではないので無効にしておく。
    fn feature() -> VoiceMemoFeature {
        VoiceMemoFeature {
            boundary_tracker: BoundaryTracker::new(0),
            ..Default::default()
        }
    }

    #[test]
    fn a_start_word_opens_a_turn() {
        let mut feature = feature();

        feature.apply_segment(1000, 2000, "じゃあターン開始します");

        assert!(feature.state.round_active);
        assert_eq!(feature.state.rounds.len(), 1);
    }

    #[test]
    fn the_turn_starts_after_the_trigger_word_itself() {
        let mut feature = feature();

        feature.apply_segment(1000, 2000, "ターン開始");

        // ワードの発話そのものはターンに含めない
        assert_eq!(feature.state.rounds[0].start_sample, 2000);
    }

    #[test]
    fn a_trigger_word_does_not_become_a_memo() {
        let mut feature = feature();

        feature.apply_segment(1000, 2000, "ターン開始");

        assert!(feature.state.rounds[0].memos.is_empty());
    }

    #[test]
    fn speech_after_the_start_word_becomes_a_memo() {
        let mut feature = feature();
        feature.apply_segment(1000, 2000, "ターン開始");

        feature.apply_segment(2000, 3000, "エレキで死体見つけた");

        assert_eq!(feature.state.rounds[0].memos.len(), 1);
    }

    #[test]
    fn an_end_word_closes_the_open_turn() {
        let mut feature = feature();
        feature.apply_segment(1000, 2000, "ターン開始");

        feature.apply_segment(5000, 6000, "ターン終了");

        assert!(!feature.state.round_active);
        assert_eq!(feature.state.rounds[0].end_sample, Some(6000));
    }
}

#[cfg(test)]
mod stale_segment_tests {
    use super::*;
    use crate::features::voice_memo::hotword::BoundaryTracker;

    /// 録音を取り直した直後の feature。前の録音は 1000 まで進んでいた。
    fn feature() -> VoiceMemoFeature {
        VoiceMemoFeature {
            boundary_tracker: BoundaryTracker::new(0),
            recording_start_sample: 1000,
            ..Default::default()
        }
    }

    #[test]
    fn a_trigger_word_from_a_previous_recording_does_not_open_a_turn() {
        let mut feature = feature();

        // 録音停止後に返ってきた前の録音のセグメント
        feature.apply_segment(100, 200, "ターン開始");

        assert!(!feature.state.round_active);
        assert!(feature.state.rounds.is_empty());
    }

    #[test]
    fn a_trigger_word_in_the_current_recording_still_opens_a_turn() {
        let mut feature = feature();

        feature.apply_segment(1000, 1200, "ターン開始");

        assert!(feature.state.round_active);
    }
}
