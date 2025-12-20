use super::{Round, VoiceMemo, VoiceMemoAction, VoiceMemoState};
use egui::{Color32, RichText, Ui};

/// 音声メモのView（純粋な描画のみ）
pub struct VoiceMemoView;

impl VoiceMemoView {
    /// UIを描画してアクションを返す
    pub fn render(state: &VoiceMemoState, ui: &mut Ui) -> Vec<VoiceMemoAction> {
        let mut actions = Vec::new();

        ui.heading("音声メモ");
        ui.separator();

        // モデルの状態表示
        if !state.model_available {
            if state.is_downloading {
                // ダウンロード中
                ui.label(RichText::new("⏳ モデルをダウンロード中...").color(Color32::YELLOW));

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
                        RichText::new("⚠ Whisperモデルが見つかりません").color(Color32::YELLOW),
                    );
                });
                ui.label("音声認識には約466MBのモデルが必要です");

                if ui.button("📥 モデルをダウンロード").clicked() {
                    actions.push(VoiceMemoAction::DownloadModel);
                }
            }
            ui.separator();
        }

        // レコーダーの状態表示
        if !state.recorder_available {
            ui.label(RichText::new("⚠ マイクが利用できません").color(Color32::RED));
            return actions;
        }

        // ラウンド進行中のタイマー表示
        if state.round_active {
            let mins = state.round_elapsed_secs as i32 / 60;
            let secs = state.round_elapsed_secs as i32 % 60;
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("⏱ {:02}:{:02}", mins, secs))
                        .size(24.0)
                        .color(Color32::LIGHT_GREEN),
                );
                if ui.button("終了").clicked() {
                    actions.push(VoiceMemoAction::EndRound);
                }
            });
        }

        // 録音コントロール
        ui.horizontal(|ui| {
            if state.is_recording {
                ui.label(
                    RichText::new(format!("● REC {:.1}秒", state.elapsed_secs))
                        .color(Color32::RED),
                );

                if ui.button("⏹ 停止").clicked() {
                    actions.push(VoiceMemoAction::StopRecording);
                }
            } else if state.is_processing {
                ui.label(RichText::new("⏳ 書き起こし中...").color(Color32::YELLOW));
            } else if state.round_active {
                // ラウンド進行中は録音ボタン
                if ui.button("🎤 録音").clicked() {
                    actions.push(VoiceMemoAction::StartRecording);
                }
            } else {
                // ラウンド開始ボタン
                if ui.button("▶ ラウンド開始").clicked() {
                    actions.push(VoiceMemoAction::StartRound);
                }

                if !state.rounds.is_empty() && ui.button("🗑 クリア").clicked() {
                    actions.push(VoiceMemoAction::ClearAllRounds);
                }
            }
        });

        // エラー表示
        if let Some(error) = &state.error {
            ui.label(RichText::new(format!("エラー: {}", error)).color(Color32::RED));
        }

        ui.separator();

        // ラウンドタブ（2件以上の時のみ表示）
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
            ui.label("ラウンドを開始してください");
        } else if let Some(round) = current_round {
            if round.memos.is_empty() {
                ui.label("録音ボタンで発言を記録");
            } else {
                ui.label(format!("メモ ({} 件)", round.memos.len()));
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for memo in &round.memos {
                            Self::render_memo(ui, memo);
                        }
                        // ラウンド終了時間を表示
                        if let Some(duration) = round.duration_secs {
                            ui.separator();
                            let mins = duration as i32 / 60;
                            let secs = duration as i32 % 60;
                            ui.label(
                                RichText::new(format!("ラウンド時間: {:02}:{:02}", mins, secs))
                                    .color(Color32::LIGHT_BLUE),
                            );
                        }
                    });
            }
        }

        actions
    }

    fn render_memo(ui: &mut Ui, memo: &VoiceMemo) {
        ui.horizontal(|ui| {
            // タイムスタンプ
            let mins = memo.timestamp_secs as i32 / 60;
            let secs = memo.timestamp_secs as i32 % 60;
            let timestamp = format!("{:02}:{:02}", mins, secs);
            ui.label(RichText::new(timestamp).monospace().color(Color32::GRAY));

            // テキスト
            ui.label(&memo.text);
        });
    }
}
