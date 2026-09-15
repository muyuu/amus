//! 書き起こしスレッドとのやり取り（リクエスト送信・結果ポーリング）

use super::hotword::{Hotword, TurnBoundary};
use super::reading;
use super::{VoiceMemo, VoiceMemoFeature};
use crate::resources::voice_recorder::SAMPLE_RATE;
use crate::resources::{RecordedSegment, TranscribeRequest};
use crate::{log_debug, log_error, log_trace};

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

            // リクエストと結果は 1:1 かつ順序が保たれるため、送信順に積んだ保留チャンクを
            // 結果を受け取った順に取り出せば対応が取れる。
            #[cfg(feature = "record-audio")]
            let pending_chunk = self.pending_audio_chunks.pop_front();

            if let Some(error) = result.error {
                log_error!("VoiceMemo", &format!("書き起こしエラー: {}", error));
                self.state.error = Some(error);
                continue;
            }

            #[cfg_attr(not(feature = "record-audio"), allow(unused_variables))]
            let kept = self.apply_segments(result.segments);

            // ターン開始の発話・ターン中の発話だと分かったチャンクだけ保存する。それ以外
            // （ターン外の雑談や誤検出）は実機データ収集として不要なので保存しない。
            #[cfg(feature = "record-audio")]
            if kept {
                if let Some((start_sample, samples)) = pending_chunk {
                    crate::resources::debug_recording::save_chunk(&samples, start_sample);
                }
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

        // record-audio: 保存するかどうかは書き起こし結果が返ってから決まるため、
        // 判断できるまで手元に残しておく（`poll_transcription_results` 参照）。
        #[cfg(feature = "record-audio")]
        self.pending_audio_chunks
            .push_back((start_sample, samples.clone()));

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

    /// 指定位置の発話をメモとして積む。実際に積んだら `true` を返す。
    ///
    /// `at` は録音上の絶対サンプル位置。その位置を含むターンへ入れ、どのターンにも
    /// 属さなければ捨てる。書き起こしはターンの開閉より遅れて返るため、所属は
    /// 送信時点ではなく位置で決める。
    pub(super) fn push_memo_at(&mut self, at: usize, text: String) -> bool {
        let Some(index) = self.round_index_at(at) else {
            return false;
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
        true
    }

    /// 指定位置を含むターンの位置。進行中のターンは終端を持たないため末尾まで含む。
    fn round_index_at(&self, at: usize) -> Option<usize> {
        self.state.rounds.iter().position(|round| {
            at >= round.start_sample && round.end_sample.is_none_or(|end| at < end)
        })
    }

    /// 1チャンク分のセグメントを取り込む。位置は録音上の絶対サンプル位置。
    ///
    /// 照合はチャンク全体の結合テキストに対して行う。セグメントは句読点で区切られる
    /// ため、トリガーワードがセグメントをまたいで割れうる（「ターン、開始」など）。
    ///
    /// トリガーワードが見つかったら語の位置で切り分ける。語自体はメモにせず、前の
    /// 発話はこれから閉じるターンへ、後ろの発話は開いたターンへ入れる。
    ///
    /// 録音を取り直した後に前の録音の結果が返ることがある。位置が現在の録音より前の
    /// セグメントは捨てる。
    ///
    /// 戻り値は、このチャンクに「ターン境界の検出」または「いずれかのターンへ積まれた
    /// 発話」があったか（`record-audio` の保存要否判断に使う。他の呼び出し側は無視してよい）。
    pub(super) fn apply_segments(&mut self, segments: Vec<RecordedSegment>) -> bool {
        let segments: Vec<RecordedSegment> = segments
            .into_iter()
            .filter(|segment| segment.start_sample >= self.recording_start_sample)
            .collect();
        if segments.is_empty() {
            return false;
        }

        // 結合テキストと、各セグメントがその何文字目から何文字目に当たるか
        let mut joined = String::new();
        let mut ranges = Vec::with_capacity(segments.len());
        for segment in &segments {
            let start = joined.chars().count();
            joined.push_str(&segment.text);
            ranges.push(start..joined.chars().count());
        }

        let matched = self.matcher.find(&joined);

        // トリガーワードが外れたときに、認識結果がどうなっていたのかを確かめるための記録。
        // 発話内容そのものなので、既定では出さず RUST_LOG=trace のときだけ出す。
        log_trace!(
            "VoiceMemo",
            &format!(
                "チャンク {}セグメント hotword={:?} text={}",
                segments.len(),
                matched.as_ref().map(|m| m.hotword),
                joined
            )
        );

        let Some(matched) = matched else {
            let mut kept = false;
            for segment in &segments {
                kept |= self.push_memo_text(segment.start_sample, &segment.text);
            }
            return kept;
        };

        let boundary = Self::sample_at(&segments, &ranges, matched.word_end);

        // 語より前の発話は、これから閉じるターンのもの。境界を動かす前に積む。
        let mut kept = false;
        for (segment, range) in segments.iter().zip(&ranges) {
            let before = take_chars(
                &segment.text,
                matched.word_start.saturating_sub(range.start),
            );
            kept |= self.push_memo_text(segment.start_sample, &before);
        }

        // トリガーワード自体の発話はターン開始/終了として意味があるため常に保存対象。
        kept |= self.move_turn_boundary(matched.hotword, boundary);

        // 語より後ろの発話は、開いたばかりのターンのもの
        for (segment, range) in segments.iter().zip(&ranges) {
            let after = skip_chars(&segment.text, matched.word_end.saturating_sub(range.start));
            kept |= self.push_memo_text(segment.start_sample.max(boundary), &after);
        }

        kept
    }

    /// 結合テキスト上の文字位置を録音上の位置へ直す。
    fn sample_at(
        segments: &[RecordedSegment],
        ranges: &[std::ops::Range<usize>],
        index: usize,
    ) -> usize {
        for (segment, range) in segments.iter().zip(ranges) {
            if index < range.end {
                let offset = index.saturating_sub(range.start);
                return Self::interpolate(
                    segment.start_sample,
                    segment.end_sample,
                    offset,
                    range.len(),
                );
            }
        }

        segments.last().map_or(0, |segment| segment.end_sample)
    }

    /// 中身があればメモとして積む。積んだら `true` を返す。
    fn push_memo_text(&mut self, at: usize, text: &str) -> bool {
        let text = reading::trim_separators(text);
        if text.is_empty() {
            return false;
        }
        self.push_memo_at(at, self.corrector.correct(text))
    }

    /// セグメント内の文字位置を録音上の位置へ直す。
    ///
    /// セグメント内のどこで何を話したかは分からないため、文字数で按分する。
    fn interpolate(start: usize, end: usize, index: usize, len: usize) -> usize {
        if len == 0 {
            return end;
        }
        start + (end - start) * index.min(len) / len
    }

    /// 検出したトリガーワードに応じてターン境界を動かす。実際に動かしたら `true` を返す
    /// （クールダウン中などで無視された場合は `false`）。
    fn move_turn_boundary(&mut self, hotword: Hotword, at: usize) -> bool {
        let Some(boundary) = self
            .boundary_tracker
            .accept(hotword, at, self.state.round_active)
        else {
            return false;
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
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::voice_memo::Round;
    use crate::resources::voice_recorder::SAMPLE_RATE;
    use crate::resources::Resources;

    #[test]
    fn restarting_transcription_drops_the_chunks_the_old_thread_held() {
        // スレッドを作り直すと処理中のチャンクの結果は返ってこない。数えたままにすると
        // 「書き起こし中」が出続ける。
        let mut feature = VoiceMemoFeature::default();
        feature.state.pending_chunks = 3;
        feature.state.is_processing = true;

        feature.restart_transcriber_thread(&Resources::without_hardware());

        assert_eq!(feature.state.pending_chunks, 0);
        assert!(!feature.state.is_processing);
    }

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
    use crate::features::voice_memo::Round;

    /// クールダウンは検証したい振る舞いではないので無効にしておく。
    fn feature() -> VoiceMemoFeature {
        VoiceMemoFeature {
            boundary_tracker: BoundaryTracker::new(0),
            ..Default::default()
        }
    }

    fn segment(start_sample: usize, end_sample: usize, text: &str) -> RecordedSegment {
        RecordedSegment {
            start_sample,
            end_sample,
            text: text.to_string(),
        }
    }

    fn memo_texts(feature: &VoiceMemoFeature, round: usize) -> Vec<&str> {
        feature.state.rounds[round]
            .memos
            .iter()
            .map(|memo| memo.text.as_str())
            .collect()
    }

    #[test]
    fn memos_are_written_in_the_known_form() {
        let mut feature = feature();
        feature.corrector = crate::features::voice_memo::vocabulary::VocabularyCorrector::new([
            "エレキ",
            "エンジン",
        ]);

        feature.apply_segments(vec![
            segment(0, 1000, "ターン開始"),
            segment(1000, 2000, "えれきからえんじんに移動"),
        ]);

        assert_eq!(memo_texts(&feature, 0), ["エレキからエンジンに移動"]);
    }

    #[test]
    fn a_start_word_opens_a_turn() {
        let mut feature = feature();

        feature.apply_segments(vec![segment(1000, 2000, "じゃあターン開始します")]);

        assert!(feature.state.round_active);
        assert_eq!(feature.state.rounds.len(), 1);
    }

    #[test]
    fn a_trigger_word_split_across_segments_is_still_found() {
        let mut feature = feature();

        // 句読点で区切られるため語が割れることがある
        feature.apply_segments(vec![
            segment(0, 1000, "はあん"),
            segment(1000, 2000, "かいし"),
        ]);

        assert!(feature.state.round_active, "分割されたワードを取り逃した");
    }

    #[test]
    fn a_trigger_word_does_not_become_a_memo() {
        let mut feature = feature();

        feature.apply_segments(vec![segment(1000, 2000, "ターン開始")]);

        assert!(feature.state.rounds[0].memos.is_empty());
    }

    #[test]
    fn the_turn_starts_right_after_the_trigger_word() {
        let mut feature = feature();

        // 10文字中5文字目「始」の直後
        feature.apply_segments(vec![segment(0, 1000, "ターン開始エンジン湧き")]);

        assert_eq!(feature.state.rounds[0].start_sample, 454);
    }

    #[test]
    fn segments_after_the_trigger_word_go_to_the_opened_turn() {
        let mut feature = feature();

        feature.apply_segments(vec![
            segment(0, 1000, "ターン開始"),
            segment(1000, 2000, "エンジン湧き"),
            segment(2000, 3000, "メイン"),
        ]);

        assert_eq!(memo_texts(&feature, 0), ["エンジン湧き", "メイン"]);
    }

    #[test]
    fn segments_before_the_trigger_word_stay_in_the_turn_being_closed() {
        let mut feature = feature();
        feature.state.rounds.push(Round {
            start_sample: 0,
            ..Default::default()
        });
        feature.state.round_active = true;

        feature.apply_segments(vec![
            segment(1000, 2000, "エレキ白"),
            segment(2000, 3000, "ターン開始"),
            segment(3000, 4000, "貨物でかえで"),
        ]);

        assert_eq!(memo_texts(&feature, 0), ["エレキ白"]);
        assert_eq!(memo_texts(&feature, 1), ["貨物でかえで"]);
    }

    #[test]
    fn each_segment_keeps_the_time_it_was_spoken() {
        let mut feature = feature();
        let second = SAMPLE_RATE as usize;

        feature.apply_segments(vec![
            segment(0, second, "ターン開始"),
            segment(second, second * 3, "エンジン湧き"),
            segment(second * 3, second * 4, "メイン"),
        ]);

        let memos = &feature.state.rounds[0].memos;
        assert_eq!(memos[0].timestamp_secs, 0.0);
        assert_eq!(memos[1].timestamp_secs, 2.0);
    }

    #[test]
    fn an_end_word_closes_the_open_turn() {
        let mut feature = feature();
        feature.apply_segments(vec![segment(1000, 2000, "ターン開始")]);

        feature.apply_segments(vec![segment(5000, 6000, "ターン終了")]);

        assert!(!feature.state.round_active);
        assert_eq!(feature.state.rounds[0].end_sample, Some(6000));
    }
}

/// `apply_segments` の戻り値（record-audio の保存要否判断に使う「保存する価値があった
/// か」）を検証する。
#[cfg(test)]
mod kept_result_tests {
    use super::*;
    use crate::features::voice_memo::hotword::BoundaryTracker;
    use crate::features::voice_memo::Round;

    fn feature() -> VoiceMemoFeature {
        VoiceMemoFeature {
            boundary_tracker: BoundaryTracker::new(0),
            ..Default::default()
        }
    }

    fn segment(start_sample: usize, end_sample: usize, text: &str) -> RecordedSegment {
        RecordedSegment {
            start_sample,
            end_sample,
            text: text.to_string(),
        }
    }

    #[test]
    fn a_turn_start_utterance_is_kept() {
        let mut feature = feature();

        let kept = feature.apply_segments(vec![segment(0, 1000, "ターン開始")]);

        assert!(kept, "ターン開始の発話は保存対象になるべき");
    }

    #[test]
    fn speech_inside_an_open_turn_is_kept() {
        let mut feature = feature();
        feature.state.rounds.push(Round {
            start_sample: 0,
            ..Default::default()
        });
        feature.state.round_active = true;

        let kept = feature.apply_segments(vec![segment(1000, 2000, "エレキ湧き")]);

        assert!(kept, "ターン中の発話は保存対象になるべき");
    }

    #[test]
    fn speech_outside_any_turn_is_not_kept() {
        let mut feature = feature();

        // ターンが一つも無い状態での雑談。トリガーワードも含まない。
        let kept = feature.apply_segments(vec![segment(0, 1000, "さっきのタスクやった")]);

        assert!(!kept, "ターン外の発話は保存対象にならないべき");
    }

    #[test]
    fn a_stale_chunk_from_a_previous_recording_is_not_kept() {
        let mut feature = VoiceMemoFeature {
            boundary_tracker: BoundaryTracker::new(0),
            recording_start_sample: 1000,
            ..Default::default()
        };

        let kept = feature.apply_segments(vec![segment(100, 200, "ターン開始")]);

        assert!(!kept, "前の録音のチャンクは保存対象にならないべき");
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

    fn segment(start_sample: usize, end_sample: usize, text: &str) -> RecordedSegment {
        RecordedSegment {
            start_sample,
            end_sample,
            text: text.to_string(),
        }
    }

    #[test]
    fn a_trigger_word_from_a_previous_recording_does_not_open_a_turn() {
        let mut feature = feature();

        // 録音停止後に返ってきた前の録音のセグメント
        feature.apply_segments(vec![segment(100, 200, "ターン開始")]);

        assert!(!feature.state.round_active);
        assert!(feature.state.rounds.is_empty());
    }

    #[test]
    fn a_trigger_word_in_the_current_recording_still_opens_a_turn() {
        let mut feature = feature();

        feature.apply_segments(vec![segment(1000, 1200, "ターン開始")]);

        assert!(feature.state.round_active);
    }
}

/// 先頭から `count` 文字。
fn take_chars(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

/// 先頭 `count` 文字を落とした残り。
fn skip_chars(text: &str, count: usize) -> String {
    text.chars().skip(count).collect()
}
