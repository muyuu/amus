use crate::common::CommonTexts;
use crate::i18n::keys::*;
use crate::state::{AppState, GameAction};
use egui::Ui;

pub struct WelcomeView;

impl WelcomeView {
    pub fn render(ui: &mut Ui, state: &AppState) -> Vec<GameAction> {
        let mut actions = Vec::new();

        let texts = WelcomeTexts::get(state);
        let common = CommonTexts::get(state);

        ui.vertical_centered(|ui| {
            ui.heading(&texts.title);
            ui.label(&texts.message);
            if ui.button(&common.button_new_game).clicked() {
                actions.push(GameAction::StartNewGame);
            }
        });

        actions
    }
}

// ウェルカムメッセージ固有のテキスト
struct WelcomeTexts {
    pub title: String,
    pub message: String,
}

impl WelcomeTexts {
    fn get(state: &AppState) -> Self {
        Self {
            title: state.t(MAIN_WELCOME_TITLE).to_string(),
            message: state.t(MAIN_WELCOME_MESSAGE).to_string(),
        }
    }
}
