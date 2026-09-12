use super::{Round, VoiceMemo, VoiceMemoAction, VoiceMemoState};
use crate::i18n::{keys as K, Translator};
use crate::resources::voice_recorder::SAMPLE_RATE;
use egui::{Color32, RichText, Ui};

/// 音声メモのView（純粋な描画のみ）
pub struct VoiceMemoView;

impl VoiceMemoView {
    /// UIを描画してアクションを返す
    pub fn render(
        state: &VoiceMemoState,
        translator: &Translator,
        ui: &mut Ui,
    ) -> Vec<VoiceMemoAction> {
        let mut actions = Vec::new();

        let max_width = ui.available_width().min(600.0);
        ui.set_max_width(max_width);

        Self::render_backend(state, ui);

        // モデルの状態表示
        if !state.model_available {
            if state.is_downloading {
                // ダウンロード中
                ui.label(
                    RichText::new(translator.t(K::VOICE_MEMO_MODEL_DOWNLOADING))
                        .color(Color32::YELLOW),
                );

                if let Some(progress) = state.download_progress {
                    ui.add(egui::ProgressBar::new(progress / 100.0).show_percentage());
                }

                // バイト数表示
                let downloaded_mb = state.downloaded_bytes as f64 / (1024.0 * 1024.0);
                if let Some(total) = state.total_bytes {
                    let total_mb = total as f64 / (1024.0 * 1024.0);
                    ui.label(format!("{:.1} MB / {:.1} MB", downloaded_mb, total_mb));
                } else {
                    ui.label(format!("{:.1} MB", downloaded_mb));
                }
            } else {
                // ダウンロード前
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(translator.t(K::VOICE_MEMO_MODEL_NOT_FOUND))
                            .color(Color32::YELLOW),
                    );
                });
                ui.label(format!(
                    "{} ({})",
                    translator.t(K::VOICE_MEMO_MODEL_REQUIRED),
                    state.model_size_label
                ));

                if ui
                    .button(translator.t(K::VOICE_MEMO_DOWNLOAD_MODEL))
                    .clicked()
                {
                    actions.push(VoiceMemoAction::DownloadModel);
                }
            }
            ui.separator();
        }

        // レコーダーの状態表示
        if !state.recorder_available {
            ui.label(
                RichText::new(translator.t(K::VOICE_MEMO_MIC_UNAVAILABLE)).color(Color32::RED),
            );
            return actions;
        }

        // ターン進行中のタイマー表示
        if state.round_active {
            let mins = state.round_elapsed_secs as i32 / 60;
            let secs = state.round_elapsed_secs as i32 % 60;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("⏱ {:02}:{:02}", mins, secs))
                        .size(24.0)
                        .color(Color32::LIGHT_GREEN),
                );
                if ui.button(translator.t(K::VOICE_MEMO_END)).clicked() {
                    actions.push(VoiceMemoAction::EndRound);
                }
            });
        }

        // ターンの操作。録音はゲーム中ずっと回っているが、内部の都合なので見せない。
        ui.horizontal(|ui| {
            if !state.round_active {
                if ui.button(translator.t(K::VOICE_MEMO_START_TURN)).clicked() {
                    actions.push(VoiceMemoAction::StartRound);
                }

                if !state.rounds.is_empty()
                    && ui.button(translator.t(K::VOICE_MEMO_CLEAR)).clicked()
                {
                    actions.push(VoiceMemoAction::ClearAllRounds);
                }
            }

            if state.is_processing {
                ui.label(
                    RichText::new(translator.t(K::VOICE_MEMO_TRANSCRIBING)).color(Color32::YELLOW),
                );
            }
        });

        // エラー表示
        if let Some(error) = &state.error {
            ui.label(
                RichText::new(format!("{}: {}", translator.t(K::VOICE_MEMO_ERROR), error))
                    .color(Color32::RED),
            );
        }

        ui.separator();

        // ターンタブ（2件以上の時のみ表示）
        if state.rounds.len() >= 2 {
            ui.horizontal_wrapped(|ui| {
                for (i, _) in state.rounds.iter().enumerate() {
                    let is_selected = i == state.selected_round;

                    let button = if is_selected {
                        egui::Button::new(RichText::new(format!("{}", i + 1)).strong())
                            .fill(Color32::from_rgb(80, 80, 120))
                    } else {
                        egui::Button::new(format!("{}", i + 1))
                    };

                    if ui.add(button).clicked() && !is_selected {
                        actions.push(VoiceMemoAction::SelectRound(i));
                    }
                }
            });
            ui.separator();
        }

        // メモ一覧
        let current_round: Option<&Round> = state.rounds.get(state.selected_round);

        if state.rounds.is_empty() {
            ui.label(translator.t(K::VOICE_MEMO_START_PROMPT));
        } else if let Some(round) = current_round {
            if round.memos.is_empty() {
                ui.label(translator.t(K::VOICE_MEMO_RECORD_HINT));
            } else {
                ui.label(format!(
                    "{} ({})",
                    translator.t(K::VOICE_MEMO_MEMO_COUNT),
                    round.memos.len()
                ));
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for memo in &round.memos {
                            Self::render_memo(ui, memo);
                        }
                        // 確定したターンの長さを表示
                        if let Some(duration) = round.end_sample.map(|end| {
                            end.saturating_sub(round.start_sample) as f32 / SAMPLE_RATE as f32
                        }) {
                            ui.separator();
                            let mins = duration as i32 / 60;
                            let secs = duration as i32 % 60;
                            ui.label(
                                RichText::new(format!(
                                    "{}: {:02}:{:02}",
                                    translator.t(K::VOICE_MEMO_TURN_TIME),
                                    mins,
                                    secs
                                ))
                                .color(Color32::LIGHT_BLUE),
                            );
                        }
                    });
            }
        }

        actions
    }

    /// GPU で動かすビルドが CPU へ退避したことを知らせる。
    ///
    /// 構成どおりなら何も出さない。GPU 版が遅いときの唯一の説明になるため、
    /// 食い違ったときだけ見せる。
    fn render_backend(state: &VoiceMemoState, ui: &mut Ui) {
        let Some(label) = &state.fallback_label else {
            return;
        };

        ui.label(
            RichText::new(format!("⚠ GPU を使えないため {} で動作中", label))
                .color(Color32::YELLOW),
        );
        ui.separator();
    }

    fn render_memo(ui: &mut Ui, memo: &VoiceMemo) {
        ui.horizontal_top(|ui| {
            // タイムスタンプ
            let mins = memo.timestamp_secs as i32 / 60;
            let secs = memo.timestamp_secs as i32 % 60;
            let timestamp = format!("{:02}:{:02}", mins, secs);
            ui.label(RichText::new(timestamp).monospace().color(Color32::GRAY));

            // テキスト（折り返しあり）
            ui.add(egui::Label::new(&memo.text).wrap());
        });
    }
}
