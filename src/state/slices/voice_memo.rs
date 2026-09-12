use crate::state::app_data::AppData;

/// 音声メモ設定の読み取り専用アクセスを提供
pub struct VoiceMemoSlice<'a> {
    data: &'a AppData,
}

impl<'a> VoiceMemoSlice<'a> {
    pub fn new(data: &'a AppData) -> Self {
        Self { data }
    }

    /// 書き起こしに GPU を使うか。
    ///
    /// ユーザーの希望であり、実際に GPU で動くとは限らない。
    pub fn use_gpu(&self) -> bool {
        self.data.use_gpu
    }
}
