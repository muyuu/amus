use egui::*;

/// 設定をまとめた構造体
pub struct WindowConf<'a> {
    /// ウィンドウタイトル（必須）
    pub title: String,

    /// 外部で管理する開閉フラグ
    /// Some(&mut bool) を渡すとユーザー操作と同期する
    /// None にすると egui 内部で開閉を管理する
    pub open: Option<&'a mut bool>,

    /// ウィンドウがリサイズ可能かどうか
    pub resizable: bool,

    /// ウィンドウが折りたたみ可能かどうか
    pub collapsible: bool,

    /// ウィンドウがドラッグ移動可能かどうか
    pub movable: bool,

    /// 初期サイズ（指定しない場合は自動）
    pub default_size: Option<Vec2>,

    /// 最小サイズ（指定しない場合は制限なし）
    pub min_size: Option<Vec2>,

    /// 最大サイズ（指定しない場合は制限なし）
    pub max_size: Option<Vec2>,

    /// 固定サイズ（指定するとリサイズ不可になる）
    pub fixed_size: Option<Vec2>,

    /// 初期位置（指定しない場合は自動）
    pub default_pos: Option<Pos2>,

    /// 画面のどこにアンカーするか（Align2 とオフセット）
    pub anchor: Option<(Align2, Vec2)>,
}

/// WindowConf を使ってウィンドウを表示するラッパ関数
pub fn base<R>(
    ctx: &Context,
    conf: WindowConf,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<InnerResponse<Option<R>>> {
    let mut win = Window::new(conf.title);

    if let Some(open) = conf.open {
        win = win.open(open);
    }
    if let Some(size) = conf.default_size {
        win = win.default_size(size);
    }
    if let Some(size) = conf.min_size {
        win = win.min_size(size);
    }
    if let Some(size) = conf.max_size {
        win = win.max_size(size);
    }
    if let Some(size) = conf.fixed_size {
        win = win.fixed_size(size);
    }
    if let Some(pos) = conf.default_pos {
        win = win.default_pos(pos);
    }
    if let Some(anchor) = conf.anchor {
        win = win.anchor(anchor.0, anchor.1);
    }

    win.resizable(conf.resizable)
        .collapsible(conf.collapsible)
        .movable(conf.movable)
        .show(ctx, add_contents)
}
