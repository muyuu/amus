use crate::common::CommonTexts;
use crate::i18n::keys::*;
use crate::state::{GameAction, Slices};
use egui::Ui;

pub struct WelcomeView;

impl WelcomeView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> Vec<GameAction> {
        let mut actions = Vec::new();

        let texts = WelcomeTexts::get(slices);
        let common = CommonTexts::get(slices);

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
    fn get(slices: &Slices<'_>) -> Self {
        Self {
            title: slices.t(MAIN_WELCOME_TITLE),
            message: slices.t(MAIN_WELCOME_MESSAGE),
        }
    }
}
