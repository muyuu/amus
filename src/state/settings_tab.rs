/// 設定モーダルのタブ。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Game,
    Display,
    Voice,
    App,
}
