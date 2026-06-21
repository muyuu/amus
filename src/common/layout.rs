use egui::*;

/// アスペクト比を計算する
fn calculate_aspect_ratio_size(available_size: Vec2, ratio: f32) -> Vec2 {
    let (width, height) = if available_size.x / available_size.y > ratio {
        // 幅が広すぎる場合、高さに合わせて幅を調整
        let height = available_size.y;
        let width = height * ratio;
        (width, height)
    } else {
        // 高さが高すぎる場合、幅に合わせて高さを調整
        let width = available_size.x;
        let height = width / ratio;
        (width, height)
    };
    Vec2::new(width, height)
}

#[allow(dead_code)]
/// アスペクト比を維持してUIを描画する
///
/// # Examples
/// ```ignore
/// aspect_ratio(ui, 16.0 / 9.0, |ui| {
///     ui.label("コンテンツ");
/// });
/// ```
pub fn aspect_ratio<R>(
    ui: &mut Ui,
    ratio: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let available_size = ui.available_size();
    let size = calculate_aspect_ratio_size(available_size, ratio);

    ui.scope(|ui| {
        ui.set_max_size(size);
        ui.set_min_size(size);
        add_contents(ui)
    })
}

/// アスペクト比を維持して中央に配置してUIを描画する
///
/// # Examples
/// ```ignore
/// aspect_ratio_centered(ui, 16.0 / 9.0, |ui| {
///     ui.label("中央配置されたコンテンツ");
/// });
/// ```
pub fn aspect_ratio_centered<R>(
    ui: &mut Ui,
    ratio: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let available_size = ui.available_size();
    let size = calculate_aspect_ratio_size(available_size, ratio);

    // 中央配置用の余白を計算
    let horizontal_margin = (available_size.x - size.x) / 2.0;
    let vertical_margin = (available_size.y - size.y) / 2.0;

    // 利用可能なスペース全体を確保して中央配置
    let (rect, _) = ui.allocate_exact_size(available_size, Sense::hover());

    // 中央配置された矩形を計算
    let centered_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + horizontal_margin, rect.min.y + vertical_margin),
        size,
    );

    // 中央配置された矩形内に子UIを作成
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(centered_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    add_contents(&mut child)
}
