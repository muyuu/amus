use super::AppState;

// 音声メモの設定
impl AppState {
    /// 書き起こしに GPU を使うか。
    ///
    /// ユーザーの希望であり、実際に GPU で動くとは限らない。ビルドに GPU バックエンドが
    /// 含まれていなければ無視される。
    pub fn use_gpu(&self) -> bool {
        self.data.use_gpu
    }

    /// 書き起こしに GPU を使うかを設定する。
    pub fn set_use_gpu(&mut self, use_gpu: bool) {
        self.data.use_gpu = use_gpu;
    }
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn gpu_is_used_by_default() {
        // GPU バックエンドを含まないビルドでは無視されるため、既定で希望しておいてよい。
        assert!(AppState::new().use_gpu());
    }

    #[test]
    fn declining_gpu_is_remembered() {
        let mut state = AppState::new();

        state.set_use_gpu(false);

        assert!(!state.use_gpu());
    }
}
