use bevy::prelude::*;

/// 统一管理UI主题资源
#[derive(Resource, Debug, Clone)]
pub struct UiTheme {
    // ── 色板 ──
    pub bg_panel:          Color,
    pub bg_sidebar:        Color,
    pub bg_content:        Color,
    pub bg_slot:           Color,
    pub text_primary:      Color,
    pub text_secondary:    Color,
    pub text_hint:         Color,
    pub border_default:    Color,
    pub border_hover:      Color,
    pub border_selected:   Color,
    pub accent:            Color,

    // ── 槽位 ──
    pub slot_size:         f32,
    pub slot_border:       f32,
    pub slot_gap:          f32,

    // ── 快捷栏 ──
    pub hotbar_height:     f32,
    pub hotbar_bg:         Color,

    // ── 分类标签 ──
    pub tab_height:        f32,
    pub tab_font_size:     f32,
    pub tab_sidebar_width: f32,
    pub tab_active_bg:     Color,
    pub tab_active_text:   Color,
    pub tab_inactive_text: Color,

    // ── 面板 ──
    pub panel_width:       f32,
    pub panel_height:      f32,
    pub panel_padding:     f32,
    pub panel_header_h:    f32,

    // ── 搜索框 ──
    pub search_width:      f32,
    pub search_height:     f32,
    pub search_font_size:  f32,
    pub search_bg:         Color,
    pub search_border:     Color,

    // ── 最近使用栏 ──
    pub recent_height:     f32,

    // ── 底部快捷栏 ──
    pub creative_hotbar_h: f32,
    pub creative_hotbar_slot: f32,

    // ── 网格 ──
    pub grid_columns:      usize,
    pub grid_padding:       f32,

    // ── 字体大小 ──
    pub title_font_size:   f32,
    pub body_font_size:    f32,
    pub small_font_size:   f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            bg_panel:          Color::srgba(0.08, 0.08, 0.10, 0.96),
            bg_sidebar:        Color::srgba(0.10, 0.10, 0.13, 1.0),
            bg_content:        Color::srgba(0.08, 0.08, 0.10, 1.0),
            bg_slot:           Color::srgba(0.12, 0.12, 0.15, 1.0),
            text_primary:      Color::srgba(0.95, 0.95, 0.97, 1.0),
            text_secondary:    Color::srgba(0.65, 0.65, 0.70, 1.0),
            text_hint:         Color::srgba(0.45, 0.45, 0.50, 1.0),
            border_default:    Color::srgba(0.20, 0.20, 0.25, 1.0),
            border_hover:      Color::srgba(0.90, 0.90, 0.30, 1.0),
            border_selected:   Color::srgba(1.0, 1.0, 1.0, 1.0),
            accent:            Color::srgba(0.35, 0.55, 0.95, 1.0),

            slot_size:         56.0,
            slot_border:       2.0,
            slot_gap:          4.0,

            hotbar_height:     60.0,
            hotbar_bg:         Color::srgba(0.08, 0.08, 0.10, 0.80),

            tab_height:        38.0,
            tab_font_size:     16.0,
            tab_sidebar_width: 160.0,
            tab_active_bg:     Color::srgba(0.20, 0.22, 0.28, 1.0),
            tab_active_text:   Color::srgba(0.95, 0.95, 0.97, 1.0),
            tab_inactive_text: Color::srgba(0.60, 0.60, 0.65, 1.0),

            panel_width:       960.0,
            panel_height:      620.0,
            panel_padding:     12.0,
            panel_header_h:    48.0,

            search_width:      200.0,
            search_height:     32.0,
            search_font_size: 14.0,
            search_bg:         Color::srgba(0.13, 0.13, 0.16, 1.0),
            search_border:     Color::srgba(0.28, 0.28, 0.33, 1.0),

            recent_height:     72.0,

            creative_hotbar_h:    80.0,
            creative_hotbar_slot: 52.0,

            grid_columns:      9,
            grid_padding:       10.0,

            title_font_size:   24.0,
            body_font_size:    14.0,
            small_font_size:   12.0,
        }
    }
}
