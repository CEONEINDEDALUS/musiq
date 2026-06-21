use eframe::egui;

// ─── Layout constants ─────────────────────────────────────────────────────────

pub const TOPBAR_HEIGHT: f32 = 32.0;
pub const DOCK_HEIGHT:   f32 = 90.0;

pub const SCRUBBER_VIS:       f32 = 3.0;
pub const SCRUBBER_VIS_HOVER: f32 = 6.0;
pub const SCRUBBER_HIT:       f32 = 16.0;

pub const ROW_HEIGHT:    f32 = 38.0;
pub const HEADER_HEIGHT: f32 = 30.0;
pub const NAV_HEIGHT:    f32 = 34.0;
pub const NAV_INDENT:    f32 = 12.0;

pub const ART_SIZE:       f32 = 44.0;
pub const PLAY_BTN:       f32 = 44.0;
pub const ICON_SIZE:      f32 = 17.0;
pub const VOL_SLIDER_W:   f32 = 80.0;
pub const VOL_SLIDER_H:   f32 = 12.0;

pub const COL_NUM_W:    f32 = 26.0;
pub const COL_ARTIST_W: f32 = 88.0;
pub const COL_DUR_W:    f32 = 44.0;
pub const COL_PLAYS_W:  f32 = 30.0;

// ─── Font sizes ───────────────────────────────────────────────────────────────

pub const TOPBAR_APP_FONT: f32 = 13.0;
pub const DIR_TOKEN_FONT:  f32 = 11.5;
pub const NAV_FONT:        f32 = 12.5;
pub const REACTOR_TITLE:   f32 = 20.0;
pub const REACTOR_META:    f32 = 12.5;
pub const REACTOR_TIME:    f32 = 11.5;

pub const DOCK_TITLE:  f32 = 12.5;
pub const DOCK_ARTIST: f32 = 10.5;
pub const TRACK_TITLE: f32 = 11.5;
pub const TRACK_HEADER: f32 = 9.5;
pub const TIME_FONT:   f32 = 9.5;

// ─── Base palette ─────────────────────────────────────────────────────────────

/// Deep dark background — almost-black with a very slight warm tint
pub const BG_BASE: egui::Color32 = egui::Color32::from_rgb(12, 12, 15);

pub const TEXT_PRIMARY:   egui::Color32 = egui::Color32::from_rgb(240, 240, 248);
pub const TEXT_SOFT:      egui::Color32 = egui::Color32::from_rgb(200, 200, 215);
pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(130, 130, 148);
pub const TEXT_DEAD:      egui::Color32 = egui::Color32::from_rgb(72, 72, 88);

/// Cyan/teal accent — the "futuristic" highlight colour
pub const ACCENT:        egui::Color32 = egui::Color32::from_rgb(0, 210, 190);
pub const ACCENT_DIM:    egui::Color32 = egui::Color32::from_rgb(0, 140, 128);
#[inline] pub fn accent_faint() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(0, 210, 190, 28) }

pub const WIN_CLOSE: egui::Color32 = egui::Color32::from_rgb(255, 95, 87);
pub const WIN_MIN:   egui::Color32 = egui::Color32::from_rgb(254, 188, 46);
pub const WIN_MAX:   egui::Color32 = egui::Color32::from_rgb(40, 200, 64);

// ─── Functional colour helpers ────────────────────────────────────────────────

#[inline] pub fn panel_glass() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(22, 22, 28, 230) }
#[inline] pub fn panel_heavy() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(18, 18, 22, 252) }
#[inline] pub fn border_subtle()  -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 12) }
#[inline] pub fn border_capsule() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(0, 210, 190, 60) }
#[inline] pub fn row_hover()  -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(0, 210, 190, 14) }
#[inline] pub fn row_active() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(0, 210, 190, 28) }
#[inline] pub fn scrubber_track() -> egui::Color32 { egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20) }

#[inline] pub fn play_btn_fill() -> egui::Color32 { ACCENT }
#[inline] pub fn play_btn_icon() -> egui::Color32 { egui::Color32::from_rgb(10, 10, 14) }

/// Active-bar accent (left edge of selected nav / track row)
#[inline] pub fn active_bar() -> egui::Color32 { ACCENT }

/// Glow: a semi-transparent accent used for art border pulse
pub fn accent_glow(alpha_0_1: f32) -> egui::Color32 {
    let a = (alpha_0_1.clamp(0.0, 1.0) * 180.0) as u8;
    egui::Color32::from_rgba_unmultiplied(0, 210, 190, a)
}

// ─── Theme application ────────────────────────────────────────────────────────

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.warn_fg_color       = TEXT_SOFT;
    visuals.error_fg_color      = TEXT_PRIMARY;
    visuals.hyperlink_color     = ACCENT;

    let mk_widget = |bg: egui::Color32, stroke_c: egui::Color32, fg: egui::Color32| {
        egui::style::WidgetVisuals {
            bg_fill:      bg,
            weak_bg_fill: bg,
            bg_stroke:    egui::Stroke::new(1.0, stroke_c),
            fg_stroke:    egui::Stroke::new(1.0, fg),
            rounding:     egui::Rounding::same(4.0),
            expansion:    0.0,
        }
    };

    visuals.widgets.noninteractive = mk_widget(BG_BASE,       border_subtle(), TEXT_SECONDARY);
    visuals.widgets.inactive       = mk_widget(panel_glass(), border_subtle(), TEXT_PRIMARY);
    visuals.widgets.hovered        = mk_widget(row_hover(),   border_capsule(), TEXT_PRIMARY);
    visuals.widgets.active         = mk_widget(row_active(),  ACCENT,           TEXT_PRIMARY);
    visuals.widgets.open           = mk_widget(panel_glass(), border_subtle(),  TEXT_PRIMARY);

    visuals.selection.bg_fill = row_active();
    visuals.selection.stroke  = egui::Stroke::new(1.0, ACCENT);

    visuals.window_fill      = BG_BASE;
    visuals.panel_fill       = BG_BASE;
    visuals.extreme_bg_color = BG_BASE;
    visuals.faint_bg_color   = panel_glass();
    visuals.code_bg_color    = panel_glass();

    visuals.indent_has_left_vline   = false;
    visuals.striped                 = false;
    visuals.slider_trailing_fill    = true;
    visuals.collapsing_header_frame = false;
    visuals.button_frame            = false;

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing      = egui::Vec2::new(6.0, 4.0);
    style.spacing.window_margin     = egui::Margin::same(8.0);
    style.spacing.button_padding    = egui::Vec2::new(0.0, 0.0);
    style.spacing.interact_size     = egui::Vec2::new(40.0, 20.0);
    style.spacing.scroll.bar_width  = 6.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.scroll.floating   = false;

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}
