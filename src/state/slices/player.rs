use crate::models::player::{Player, PlayerId};
use crate::state::app_data::AppData;

/// プレイヤー関連の読み取り専用アクセスを提供
pub struct PlayerSlice<'a> {
    data: &'a AppData,
}

impl<'a> PlayerSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 全プレイヤーへの参照を取得
    pub fn players(&self) -> Option<&'a [Player]> {
        self.data.game.as_ref().map(|g| g.players.as_slice())
    }

    /// 指定IDのプレイヤーへの参照を取得
    pub fn player(&self, player_id: PlayerId) -> Option<&'a Player> {
        self.players()?.iter().find(|p| p.id == player_id)
    }

    /// 選択中のプレイヤーID
    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.selected_player_id
    }

    /// 指定プレイヤーが選択中かどうか
    pub fn is_selected(&self, player_id: PlayerId) -> bool {
        self.data.selected_player_id == Some(player_id)
    }

    /// 指定プレイヤーが名前編集中かどうか
    pub fn is_editing_name(&self, player_id: PlayerId) -> bool {
        self.data.editing_name_player_id == Some(player_id)
    }
}
