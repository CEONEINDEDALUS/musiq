use eframe::egui;

use crate::app::{MusiqApp, NavSection};
use crate::audio::PlayState;
use crate::search;
use crate::theme::*;

// ─── Welcome screen ───────────────────────────────────────────────────────────

pub fn welcome(app: &mut MusiqApp, ctx: &egui::Context) {
    titlebar(app, ctx);

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(BG_BASE))
        .show(ctx, |ui| {
            let any_hover = ui.input(|i| !i.raw.hovered_files.is_empty());
            let avail_w   = ui.available_width();
            let input_w   = (avail_w * 0.6).clamp(280.0, 480.0);

            // Subtle radial glow behind the logo
            let painter = ui.painter();
            let center  = ui.max_rect().center();
            painter.circle_filled(
                center - egui::Vec2::new(0.0, 60.0),
                avail_w * 0.25,
                egui::Color32::from_rgba_unmultiplied(0, 210, 190, 6),
            );

            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.13);

                if app.scanning {
                    // ── Scanning state ──────────────────────────────────────
                    let dots = match ((app.viz_phase * 1.2) as usize) % 4 {
                        0 => "",  1 => "·",  2 => "··",  _ => "···",
                    };
                    ui.label(egui::RichText::new("scanning library").size(22.0).color(ACCENT));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(format!("reading tags{}", dots))
                        .size(12.0).color(TEXT_DEAD).family(egui::FontFamily::Monospace));
                } else {
                    // ── Logo ────────────────────────────────────────────────
                    // "musiq" with accent-coloured 'q'
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                        // Centre manually
                        let label_w = 220.0; // approx
                        let pad = ((avail_w - label_w) / 2.0).max(0.0);
                        ui.add_space(pad);
                        ui.label(egui::RichText::new("musi").size(52.0).strong().color(TEXT_PRIMARY));
                        ui.label(egui::RichText::new("q").size(52.0).strong().color(ACCENT));
                    });
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("a minimal music player for linux")
                        .size(12.0).color(TEXT_DEAD));
                    ui.add_space(32.0);

                    // Path input
                    let resp = ui.add_sized(
                        [input_w, 36.0],
                        egui::TextEdit::singleline(&mut app.input)
                            .hint_text(
                                egui::RichText::new("/path/to/your/music").color(TEXT_DEAD),
                            )
                            .text_color(TEXT_PRIMARY)
                            .background_color(panel_glass())
                            .margin(egui::Margin::symmetric(14.0, 8.0))
                            .font(egui::FontId::monospace(DIR_TOKEN_FONT)),
                    );
                    if !resp.has_focus() && ctx.memory(|m| m.focused().is_none()) {
                        resp.request_focus();
                    }
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        submit_welcome_path(app);
                    }

                    ui.add_space(12.0);

                    let btn_w = (input_w * 0.55).clamp(160.0, 240.0);
                    let open_btn = egui::Button::new(
                        egui::RichText::new("open music folder").size(12.5).color(play_btn_icon()),
                    )
                    .fill(ACCENT)
                    .stroke(egui::Stroke::NONE)
                    .rounding(egui::Rounding::same(6.0))
                    .min_size(egui::Vec2::new(btn_w, 38.0));
                    if ui.add(open_btn).clicked() {
                        app.open_folder_dialog();
                    }

                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("or drag & drop a folder anywhere")
                        .size(11.0).color(TEXT_DEAD));

                    if any_hover {
                        ui.add_space(20.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(0, 210, 190, 12))
                            .stroke(egui::Stroke::new(2.0, ACCENT))
                            .rounding(egui::Rounding::same(10.0))
                            .inner_margin(egui::Margin::same(18.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("⬇  drop folder to scan")
                                    .size(18.0).color(ACCENT));
                            });
                    }

                    // Recent folders
                    if !app.recent_folders.is_empty() {
                        ui.add_space(36.0);
                        ui.label(egui::RichText::new("RECENT")
                            .size(9.5).strong().color(TEXT_DEAD)
                            .extra_letter_spacing(1.5));
                        ui.add_space(8.0);
                        let recent: Vec<_> = app.recent_folders.iter().take(6).cloned().collect();
                        for folder in recent {
                            let name = folder.file_name()
                                .and_then(|n| n.to_str()).unwrap_or("?");
                            let btn_w = (input_w * 0.8).clamp(200.0, 380.0);
                            let btn = egui::Button::new(
                                egui::RichText::new(format!("⌂  {}", name))
                                    .size(11.5).color(TEXT_SECONDARY),
                            )
                            .fill(panel_glass())
                            .stroke(egui::Stroke::new(1.0, border_capsule()))
                            .rounding(egui::Rounding::same(6.0))
                            .min_size(egui::Vec2::new(btn_w, 30.0));
                            if ui.add(btn).clicked() {
                                app.scan_path(folder);
                            }
                            ui.add_space(3.0);
                        }
                    }
                }
            });
        });
}

// ─── Player screen ────────────────────────────────────────────────────────────

pub fn player(app: &mut MusiqApp, ctx: &egui::Context) {
    titlebar(app, ctx);
    dock(app, ctx);

    let total_w = ctx.screen_rect().width();
    let left_w  = (total_w * 0.22).clamp(140.0, 220.0);
    let right_w = (total_w * 0.38).clamp(220.0, 480.0);

    egui::SidePanel::left("left_col")
        .resizable(false)
        .default_width(left_w)
        .width_range(120.0..=left_w)
        .frame(
            egui::Frame::none()
                .fill(panel_glass())
                .stroke(egui::Stroke::new(1.0, border_subtle())),
        )
        .show(ctx, |ui| left_column(app, ui));

    egui::SidePanel::right("right_col")
        .resizable(false)
        .default_width(right_w)
        .width_range(right_w..=right_w)
        .frame(
            egui::Frame::none()
                .fill(panel_glass())
                .stroke(egui::Stroke::new(1.0, border_subtle())),
        )
        .show(ctx, |ui| right_column(app, ctx, ui));

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(panel_glass()))
        .show(ctx, |ui| center_column(app, ctx, ui));
}

// ─── Title bar ────────────────────────────────────────────────────────────────

fn titlebar(app: &mut MusiqApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("titlebar")
        .resizable(false)
        .exact_height(TOPBAR_HEIGHT)
        .frame(egui::Frame::none().fill(BG_BASE))
        .show(ctx, |ui| {
            let full_w = ui.available_width();
            let (rect, response) = ui.allocate_exact_size(
                egui::Vec2::new(full_w, TOPBAR_HEIGHT),
                egui::Sense::click_and_drag(),
            );

            if response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if response.double_clicked() {
                app.is_maximized = !app.is_maximized;
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(app.is_maximized));
            }

            ui.allocate_ui_at_rect(rect, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(14.0);
                    if traffic_dot(ui, WIN_CLOSE).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    ui.add_space(6.0);
                    if traffic_dot(ui, WIN_MIN).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    ui.add_space(6.0);
                    if traffic_dot(ui, WIN_MAX).clicked() {
                        app.is_maximized = !app.is_maximized;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(app.is_maximized));
                    }

                    let avail    = ui.available_width();
                    let label_w  = 60.0;
                    let pad      = (avail - label_w) / 2.0;
                    if pad > 0.0 { ui.add_space(pad); }
                    // "musi" + accent "q"
                    ui.label(egui::RichText::new("musi")
                        .size(TOPBAR_APP_FONT).color(TEXT_SOFT).extra_letter_spacing(0.3));
                    ui.label(egui::RichText::new("q")
                        .size(TOPBAR_APP_FONT).color(ACCENT).extra_letter_spacing(0.3));
                });
            });
        });
}

fn traffic_dot(ui: &mut egui::Ui, color: egui::Color32) -> egui::Response {
    let btn = egui::Button::new("")
        .fill(color)
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(6.0))
        .min_size(egui::Vec2::new(12.0, 12.0));
    ui.add(btn)
}

// ─── Left column — nav ────────────────────────────────────────────────────────

fn left_column(app: &mut MusiqApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                ui.add_space(NAV_INDENT);
                let folder_text = app.library_folder
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "select folder".to_string());
                let avail = (ui.available_width() - NAV_INDENT).max(60.0);
                let btn = egui::Button::new(
                    egui::RichText::new(format!("⌂  {}", truncate_path(&folder_text, 16)))
                        .size(DIR_TOKEN_FONT)
                        .color(ACCENT)
                        .family(egui::FontFamily::Monospace),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, border_capsule()))
                .rounding(egui::Rounding::same(999.0))
                .min_size(egui::Vec2::new(avail, 30.0));
                if ui.add(btn).clicked() {
                    app.open_folder_dialog();
                }
            });

            ui.add_space(16.0);

            nav_item(ui, app, "Tracks",    NavSection::Tracks,    "≡");
            nav_item(ui, app, "Albums",    NavSection::Albums,    "◈");
            nav_item(ui, app, "Artists",   NavSection::Artists,   "♪");
            nav_item(ui, app, "Favorites", NavSection::Favorites, "♡");

            let pl_expanded = app.nav_expanded && app.nav_selected == NavSection::Playlists;
            nav_item(ui, app, "Playlists", NavSection::Playlists,
                if pl_expanded { "▾" } else { "▸" });
            if pl_expanded {
                for pl in app.playlists.clone() {
                    playlist_item(ui, &pl);
                }
            }

            nav_item(ui, app, "Music", NavSection::Music, "◈");

            ui.add_space(20.0);

            if !app.library.tracks.is_empty() {
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(NAV_INDENT, 6.0))
                    .show(ui, |ui| {
                        let dur = app.library.total_duration_secs();
                        let h   = dur / 3600;
                        let m   = (dur % 3600) / 60;
                        ui.label(
                            egui::RichText::new(format!(
                                "{} tracks · {}h {}m",
                                app.library.tracks.len(), h, m
                            ))
                            .size(10.0)
                            .color(TEXT_DEAD)
                            .family(egui::FontFamily::Monospace),
                        );
                    });
            }
        });
}

fn nav_item(ui: &mut egui::Ui, app: &mut MusiqApp, label: &str, section: NavSection, icon: &str) {
    let is_active = app.nav_selected == section;
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(ui.available_width(), NAV_HEIGHT),
        egui::Sense::click(),
    );

    let fill = if is_active { row_active() } else if response.hovered() { row_hover() } else { egui::Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, egui::Rounding::ZERO, fill);

    if is_active {
        let bar = egui::Rect::from_min_size(rect.min, egui::Vec2::new(2.0, rect.height()));
        ui.painter().rect_filled(bar, egui::Rounding::ZERO, active_bar());
    }

    let text_color = if is_active { ACCENT } else if response.hovered() { TEXT_SOFT } else { TEXT_SECONDARY };
    let icon_color = if is_active { ACCENT } else if response.hovered() { TEXT_SOFT } else { TEXT_DEAD };

    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.add_space(NAV_INDENT + 4.0);
    child.label(egui::RichText::new(icon).size(15.0).color(icon_color));
    child.add_space(8.0);
    child.label(egui::RichText::new(label).size(NAV_FONT).color(text_color));

    if response.clicked() {
        if section == NavSection::Playlists {
            if app.nav_selected == NavSection::Playlists {
                app.nav_expanded = !app.nav_expanded;
            } else {
                app.nav_selected = section;
                app.nav_expanded = true;
            }
        } else {
            app.nav_selected = section;
        }
    }
}

fn playlist_item(ui: &mut egui::Ui, name: &str) {
    let row_h = NAV_HEIGHT * 0.85;
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(ui.available_width(), row_h),
        egui::Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_filled(rect, egui::Rounding::ZERO, row_hover());
    }
    let text_color = if response.hovered() { TEXT_SOFT } else { TEXT_SECONDARY };
    let mut child  = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.add_space(NAV_INDENT + 24.0);
    child.label(egui::RichText::new("♫").size(12.0)
        .color(if response.hovered() { ACCENT_DIM } else { TEXT_DEAD }));
    child.add_space(8.0);
    child.label(egui::RichText::new(name).size(NAV_FONT - 0.5).color(text_color));
}

// ─── Center column — album art + now-playing ──────────────────────────────────

fn center_column(app: &mut MusiqApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let avail = ui.available_rect_before_wrap();

    ui.vertical_centered(|ui| {
        ui.add_space((avail.height() * 0.06).max(8.0));

        let art_size = (avail.width() * 0.55).clamp(120.0, 300.0);
        let (art_rect, _) = ui.allocate_exact_size(egui::Vec2::splat(art_size), egui::Sense::hover());
        let painter = ui.painter_at(art_rect);

        // ── Animated glow ring when playing ──────────────────────────────
        if app.engine.state == PlayState::Playing {
            let phase   = app.viz_phase;
            let pulse   = (phase.sin() * 0.5 + 0.5) as f32;         // 0..1
            let glow_r  = art_size / 2.0 + 4.0 + pulse * 6.0;
            let glow_a  = (0.15 + pulse * 0.25) as f32;
            painter.circle_stroke(
                art_rect.center(),
                glow_r,
                egui::Stroke::new(2.0 + pulse * 2.0, accent_glow(glow_a)),
            );
            // Second outer ring, offset phase
            let pulse2  = ((phase + 1.5).sin() * 0.5 + 0.5) as f32;
            painter.circle_stroke(
                art_rect.center(),
                art_size / 2.0 + 10.0 + pulse2 * 8.0,
                egui::Stroke::new(1.0, accent_glow(0.06 + pulse2 * 0.08)),
            );
        }

        let art_rounding = egui::Rounding::same(10.0);

        if let Some(idx) = app.current_track_idx {
            if let Some(tex) = app.ensure_album_art(ctx, idx) {
                painter.image(
                    tex.id(), art_rect.shrink(1.0),
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
                // Accent border overlay
                painter.rect_stroke(art_rect, art_rounding,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 210, 190, 40)));
            } else {
                painter.rect_filled(art_rect, art_rounding, panel_heavy());
                painter.rect_stroke(art_rect, art_rounding,
                    egui::Stroke::new(1.0, border_subtle()));
                painter.text(art_rect.center(), egui::Align2::CENTER_CENTER, "♪",
                    egui::FontId::proportional(art_size * 0.25), ACCENT_DIM);
            }
        } else {
            painter.rect_filled(art_rect, art_rounding, panel_heavy());
            painter.rect_stroke(art_rect, art_rounding,
                egui::Stroke::new(1.0, border_subtle()));
            painter.text(art_rect.center(), egui::Align2::CENTER_CENTER, "♪",
                egui::FontId::proportional(art_size * 0.25), TEXT_DEAD);
        }

        ui.add_space(22.0);

        if let Some(track) = app.current_track() {
            let title     = if track.title.is_empty() { "—" } else { &track.title };
            let max_chars = ((avail.width() / 8.5) as usize).clamp(12, 36);
            ui.label(egui::RichText::new(truncate(title, max_chars))
                .size(REACTOR_TITLE).strong().color(TEXT_PRIMARY));
            ui.add_space(4.0);
            ui.label(egui::RichText::new(truncate(&track.artist, max_chars + 4))
                .size(REACTOR_META).color(TEXT_SECONDARY));
            ui.add_space(2.0);
            ui.label(egui::RichText::new(truncate(&track.album, max_chars + 4))
                .size(REACTOR_META).color(TEXT_DEAD));
            ui.add_space(10.0);

            // Time display
            let (elapsed, total) = app.engine.position();
            ui.label(
                egui::RichText::new(format!("{} / {}", fmt_time(elapsed), fmt_time(total)))
                    .size(REACTOR_TIME)
                    .family(egui::FontFamily::Monospace)
                    .color(TEXT_DEAD),
            );
            ui.add_space(8.0);

            // Format + bitrate badge
            egui::Frame::none()
                .fill(accent_faint())
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 210, 190, 40)))
                .rounding(egui::Rounding::same(4.0))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}  {}", track.format, track.bitrate_display()))
                            .size(9.5)
                            .color(ACCENT_DIM)
                            .family(egui::FontFamily::Monospace),
                    );
                });

            // Play count (if > 0)
            if let Some(idx) = app.current_track_idx {
                let plays = app.play_count(idx);
                if plays > 0 {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!("played {} time{}", plays,
                            if plays == 1 { "" } else { "s" }))
                            .size(9.5)
                            .color(TEXT_DEAD)
                            .family(egui::FontFamily::Monospace),
                    );
                }
            }
        } else if app.scanning {
            let dots = match ((app.viz_phase * 1.2) as usize) % 4 {
                0 => "",  1 => "·",  2 => "··",  _ => "···",
            };
            ui.label(egui::RichText::new(format!("scanning{}", dots))
                .size(REACTOR_TITLE).color(ACCENT));
        } else {
            ui.label(egui::RichText::new("no track selected").size(REACTOR_TITLE).color(TEXT_DEAD));
            ui.add_space(6.0);
            ui.label(egui::RichText::new("open a folder or drop one here")
                .size(REACTOR_META).color(TEXT_DEAD));
        }
    });
}

// ─── Right column — track list ────────────────────────────────────────────────

fn right_column(app: &mut MusiqApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let col_w      = ui.available_width();
    let scrollbar_w = 6.0;
    let inner_w    = col_w - scrollbar_w;

    let has_query = !app.search_query.trim().is_empty();

    // ── Search bar ──────────────────────────────────────────────────────────
    let search_h = 44.0;
    let (search_rect, _) = ui.allocate_exact_size(
        egui::Vec2::new(col_w, search_h),
        egui::Sense::hover(),
    );
    let s_bg = egui::Rect::from_min_size(
        search_rect.min + egui::Vec2::new(10.0, 7.0),
        egui::Vec2::new(col_w - 20.0, 30.0),
    );
    let sp = ui.painter().with_clip_rect(search_rect);
    sp.rect_filled(s_bg, egui::Rounding::same(8.0), panel_glass());
    // Glow border when search active
    let search_stroke_color = if has_query {
        egui::Color32::from_rgba_unmultiplied(0, 210, 190, 100)
    } else {
        border_subtle()
    };
    sp.rect_stroke(s_bg, egui::Rounding::same(8.0), egui::Stroke::new(1.0, search_stroke_color));

    let mut s_child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(s_bg)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    s_child.add_space(10.0);
    let search_icon_color = if has_query { ACCENT } else { TEXT_DEAD };
    s_child.label(egui::RichText::new("⌕").size(13.0).color(search_icon_color));
    s_child.add_space(4.0);

    let text_w = if has_query { s_bg.width() - 62.0 } else { s_bg.width() - 32.0 };
    let search_resp = s_child.add_sized(
        [text_w, 30.0],
        egui::TextEdit::singleline(&mut app.search_query)
            .hint_text(
                egui::RichText::new("search title · artist · album · path")
                    .color(TEXT_DEAD)
                    .family(egui::FontFamily::Monospace),
            )
            .text_color(TEXT_PRIMARY)
            .frame(false)
            .margin(egui::Margin::symmetric(0.0, 6.0))
            .font(egui::FontId::monospace(11.0)),
    );

    if search_resp.has_focus() && s_child.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.search_query.clear();
    }

    if has_query {
        let (cr, cr_resp) = s_child.allocate_exact_size(egui::Vec2::splat(20.0), egui::Sense::click());
        if cr_resp.hovered() {
            s_child.painter().rect_filled(cr, egui::Rounding::same(10.0), row_hover());
        }
        s_child.painter().text(cr.center(), egui::Align2::CENTER_CENTER, "✕",
            egui::FontId::proportional(11.0), TEXT_SECONDARY);
        if cr_resp.clicked() { app.search_query.clear(); }
    }

    // ── Column header ───────────────────────────────────────────────────────
    let (header_rect, _) = ui.allocate_exact_size(
        egui::Vec2::new(col_w, HEADER_HEIGHT),
        egui::Sense::hover(),
    );
    let hp   = ui.painter().with_clip_rect(header_rect);
    let hpad = 12.0;
    let cy   = header_rect.center().y;
    hp.text(egui::Pos2::new(header_rect.left() + hpad, cy),
        egui::Align2::LEFT_CENTER, "#",
        egui::FontId::monospace(TRACK_HEADER), TEXT_DEAD);
    hp.text(egui::Pos2::new(header_rect.left() + hpad + COL_NUM_W, cy),
        egui::Align2::LEFT_CENTER, "TITLE",
        egui::FontId::monospace(TRACK_HEADER), TEXT_DEAD);
    let aw = col_artist_w(inner_w);
    let artist_x = header_rect.right() - scrollbar_w - hpad - COL_DUR_W - 6.0 - aw;
    if inner_w > 260.0 {
        hp.text(egui::Pos2::new(artist_x, cy),
            egui::Align2::LEFT_CENTER, "ARTIST",
            egui::FontId::monospace(TRACK_HEADER), TEXT_DEAD);
    }
    hp.text(egui::Pos2::new(header_rect.right() - scrollbar_w - hpad, cy),
        egui::Align2::RIGHT_CENTER, "TIME",
        egui::FontId::monospace(TRACK_HEADER), TEXT_DEAD);
    hp.line_segment(
        [egui::Pos2::new(header_rect.left(), header_rect.bottom() - 0.5),
         egui::Pos2::new(header_rect.right(), header_rect.bottom() - 0.5)],
        egui::Stroke::new(1.0, border_subtle()),
    );

    // ── Track count badge ───────────────────────────────────────────────────
    let track_indices = app.filtered_tracks();
    let total_count   = app.library.tracks.len();

    let count_str = if track_indices.is_empty() {
        if has_query { format!("no matches · {} tracks", total_count) }
        else         { "no tracks".to_string() }
    } else if has_query {
        format!("{} of {} tracks", track_indices.len(), total_count)
    } else {
        format!("{} tracks", total_count)
    };
    ui.allocate_ui_with_layout(
        egui::Vec2::new(col_w, 20.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_space(hpad);
            let count_color = if has_query { ACCENT_DIM } else { TEXT_DEAD };
            ui.label(egui::RichText::new(&count_str).size(10.0).color(count_color)
                .family(egui::FontFamily::Monospace));
        },
    );

    if track_indices.is_empty() {
        ui.add_space(24.0);
        ui.horizontal(|ui| {
            ui.add_space(hpad);
            ui.label(egui::RichText::new(if has_query {
                "no matches — try a shorter search"
            } else {
                "drop a folder or click the path pill"
            }).size(11.0).color(TEXT_DEAD));
        });
        return;
    }

    // ── Track rows ──────────────────────────────────────────────────────────
    let row_h   = ROW_HEIGHT;
    let query   = app.search_query.clone();
    let current = app.current_track_idx;

    let scroll_to = app.scroll_to_track.take();

    let search_has_focus = ctx.memory(|m| m.focused().is_some());
    if !search_has_focus {
        let move_by: i32 = ctx.input(|i| {
            if i.key_pressed(egui::Key::J) { 1 }
            else if i.key_pressed(egui::Key::K) { -1 }
            else { 0 }
        });
        if move_by != 0 {
            let new_idx = if let Some(cur) = current {
                let pos  = track_indices.iter().position(|&x| x == cur).unwrap_or(0);
                let next = (pos as i32 + move_by).clamp(0, track_indices.len() as i32 - 1) as usize;
                track_indices[next]
            } else {
                track_indices[0]
            };
            app.select_track(new_idx);
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .show(ui, |ui| {
            ui.set_min_size(egui::Vec2::new(inner_w, ui.available_height()));

            for (display_num, &idx) in track_indices.iter().enumerate() {
                if idx >= app.library.tracks.len() { continue; }

                let track        = &app.library.tracks[idx];
                let track_title  = track.title.clone();
                let track_artist = track.artist.clone();
                let track_album  = track.album.clone();
                let track_format = track.format.clone();
                let track_bitrate = track.bitrate_display();
                let track_dur    = track.duration_display();
                let track_path   = track.path.display().to_string();
                let is_current   = current == Some(idx);
                let play_count   = app.persist.play_count_for(&track.path);

                let (rect, response) = ui.allocate_exact_size(
                    egui::Vec2::new(inner_w, row_h),
                    egui::Sense::click(),
                );

                if is_current {
                    if let Some(target_idx) = scroll_to {
                        if target_idx == idx {
                            ui.scroll_to_rect(rect, Some(egui::Align::Center));
                        }
                    }
                }

                let fill = if is_current { row_active() } else if response.hovered() { row_hover() } else { egui::Color32::TRANSPARENT };
                ui.painter().rect_filled(rect, egui::Rounding::ZERO, fill);

                if is_current {
                    let bar = egui::Rect::from_min_size(rect.min, egui::Vec2::new(2.0, rect.height()));
                    ui.painter().rect_filled(bar, egui::Rounding::ZERO, active_bar());
                }

                let painter  = ui.painter_at(rect);
                let center_y = rect.center().y;
                let lpad     = 14.0;
                let rpad     = 12.0;
                let row_w    = rect.width();
                let aw       = col_artist_w(inner_w);

                // Track number / playing indicator
                let num_text  = if is_current { "▶".to_string() } else { format!("{:>2}", display_num + 1) };
                let num_color = if is_current { ACCENT } else { TEXT_DEAD };
                painter.text(
                    egui::Pos2::new(rect.left() + lpad, center_y),
                    egui::Align2::LEFT_CENTER, &num_text,
                    egui::FontId::monospace(TRACK_TITLE), num_color,
                );

                // Title with search highlighting
                let title_x     = rect.left() + lpad + COL_NUM_W;
                let title_max_w = (row_w - lpad - COL_NUM_W - aw - COL_DUR_W - rpad - 8.0).max(40.0);
                let title_color = if is_current { TEXT_PRIMARY } else if response.hovered() { TEXT_SOFT } else { TEXT_SECONDARY };
                let max_title_chars = ((title_max_w / TRACK_TITLE * 1.35) as usize).clamp(8, 80);
                let job = build_title_job(
                    &truncate(&track_title, max_title_chars),
                    &query, title_color, ACCENT, row_active(),
                    egui::FontId::monospace(TRACK_TITLE),
                );
                let galley = ctx.fonts(|f| f.layout_job(job));
                let gs     = galley.size();
                painter.galley(egui::Pos2::new(title_x, center_y - gs.y / 2.0), galley, egui::Color32::WHITE);

                // Artist
                if inner_w > 260.0 {
                    let artist_col_x = rect.right() - rpad - COL_DUR_W - 6.0 - aw;
                    let artist_color = if is_current { TEXT_SECONDARY } else { TEXT_DEAD };
                    let max_ac = ((aw / TRACK_TITLE * 1.35) as usize).clamp(4, 24);
                    painter.text(
                        egui::Pos2::new(artist_col_x, center_y),
                        egui::Align2::LEFT_CENTER,
                        truncate(&track_artist, max_ac),
                        egui::FontId::monospace(TRACK_TITLE),
                        artist_color,
                    );
                }

                // Duration
                let dur_color = if is_current { ACCENT_DIM } else { TEXT_DEAD };
                painter.text(
                    egui::Pos2::new(rect.right() - rpad, center_y),
                    egui::Align2::RIGHT_CENTER, &track_dur,
                    egui::FontId::monospace(TRACK_TITLE), dur_color,
                );

                // Tiny play-count dot badge (top-right of row when > 0)
                if play_count > 0 {
                    let badge_x = rect.right() - rpad - 2.0;
                    let badge_y = rect.top() + 5.0;
                    let badge_r = 3.0f32.min(play_count as f32 * 0.8 + 1.5).clamp(1.5, 4.0);
                    painter.circle_filled(
                        egui::Pos2::new(badge_x, badge_y),
                        badge_r,
                        egui::Color32::from_rgba_unmultiplied(0, 210, 190, 140),
                    );
                }

                let tip = format!(
                    "{}\n{} · {}\n{} · {}\n{}", track_title, track_artist, track_album,
                    track_format, track_bitrate, track_path,
                );
                response.clone().on_hover_text(tip);

                if response.clicked() {
                    app.select_track(idx);
                }
            }
        });
}

/// Responsive artist column width
fn col_artist_w(col_w: f32) -> f32 {
    if col_w < 260.0      { 0.0 }
    else if col_w < 340.0 { 60.0 }
    else if col_w < 440.0 { 80.0 }
    else                  { COL_ARTIST_W }
}

fn build_title_job(
    title: &str, query: &str,
    normal: egui::Color32, hi: egui::Color32, hi_bg: egui::Color32,
    font: egui::FontId,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let q = query.trim();
    if q.is_empty() {
        job.append(title, 0.0, egui::text::TextFormat { font_id: font, color: normal, ..Default::default() });
        return job;
    }
    // Use highlight ranges from search module
    let ranges = search::highlight_ranges(title, q);
    if ranges.is_empty() {
        job.append(title, 0.0, egui::text::TextFormat { font_id: font, color: normal, ..Default::default() });
        return job;
    }
    let mut pos = 0usize;
    for (start, end) in &ranges {
        let start = (*start).min(title.len());
        let end   = (*end).min(title.len());
        if pos < start {
            job.append(&title[pos..start], 0.0, egui::text::TextFormat { font_id: font.clone(), color: normal, ..Default::default() });
        }
        if start < end {
            job.append(&title[start..end], 0.0, egui::text::TextFormat { font_id: font.clone(), color: hi, background: hi_bg, ..Default::default() });
        }
        pos = end;
    }
    if pos < title.len() {
        job.append(&title[pos..], 0.0, egui::text::TextFormat { font_id: font, color: normal, ..Default::default() });
    }
    job
}

// ─── Dock ─────────────────────────────────────────────────────────────────────

fn dock(app: &mut MusiqApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("dock")
        .resizable(false)
        .exact_height(DOCK_HEIGHT)
        .frame(
            egui::Frame::none()
                .fill(panel_heavy())
                .stroke(egui::Stroke::new(1.0, border_subtle())),
        )
        .show(ctx, |ui| {
            let w = ui.available_width();

            // Top half: track info + controls + volume
            ui.allocate_ui_with_layout(
                egui::Vec2::new(w, 50.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_space(10.0);

                    let left_w  = (w * 0.27).clamp(140.0, 240.0);
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(left_w, 50.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| { dock_art(app, ctx, ui); },
                    );

                    let right_w  = (w * 0.28).clamp(120.0, 220.0);
                    let center_w = w - left_w - right_w - 20.0;
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(center_w, 50.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::Vec2::new(14.0, 0.0);
                                if transport_icon(ui, "⇄", app.engine.shuffle, "Shuffle (S)").clicked() { app.toggle_shuffle(); }
                                if transport_icon(ui, "⏮", false, "Previous (←)").clicked() { app.play_previous(); }
                                if play_button(ui, app.engine.state == PlayState::Playing).clicked() { app.play_pause(); }
                                if transport_icon(ui, "⏭", false, "Next (→)").clicked() { app.play_next(); }
                                if transport_icon(ui, "↻", app.engine.repeat, "Repeat (R)").clicked() { app.toggle_repeat(); }
                            });
                        },
                    );

                    // Volume
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(ui.available_width() - 10.0, 50.0),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(format!("{:>3}%", (app.volume * 100.0) as i32))
                                    .size(9.5).color(TEXT_DEAD)
                                    .family(egui::FontFamily::Monospace),
                            );
                            ui.add_space(4.0);
                            let vol_icon = if app.volume == 0.0 { "🔇" } else if app.volume < 0.4 { "🔉" } else { "🔊" };
                            ui.label(egui::RichText::new(vol_icon).size(13.0).color(TEXT_SECONDARY));
                            ui.add_space(6.0);
                            let mut vol = app.volume;
                            let resp = ui.add_sized(
                                [VOL_SLIDER_W, VOL_SLIDER_H],
                                egui::Slider::new(&mut vol, 0.0..=1.0)
                                    .show_value(false)
                                    .step_by(0.01)
                                    .trailing_fill(true),
                            );
                            if resp.changed() { app.set_volume(vol); }
                        },
                    );
                },
            );

            // Scrubber row
            let scrub_w = w - 24.0;
            ui.allocate_ui_with_layout(
                egui::Vec2::new(w, 20.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add_space(12.0);
                    scrubber(ui, app, scrub_w - 60.0);
                    ui.add_space(6.0);
                    let (elapsed, total) = app.engine.position();
                    ui.label(
                        egui::RichText::new(format!("{}/{}", fmt_time(elapsed), fmt_time(total)))
                            .size(TIME_FONT)
                            .family(egui::FontFamily::Monospace)
                            .color(TEXT_SECONDARY),
                    );
                },
            );

            // Status toast
            if let Some((msg, _)) = &app.status_message {
                let msg = msg.clone();
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(w, 12.0),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label(
                            egui::RichText::new(msg).size(9.5).color(ACCENT_DIM)
                                .family(egui::FontFamily::Monospace),
                        );
                    },
                );
            }
        });
}

fn dock_art(app: &mut MusiqApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(ART_SIZE), egui::Sense::hover());
    let painter   = ui.painter_at(rect);
    let art_rect  = rect.shrink(1.0);

    if let Some(idx) = app.current_track_idx {
        if let Some(tex) = app.ensure_album_art(ctx, idx) {
            painter.image(
                tex.id(), art_rect,
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            // Thin accent border
            painter.rect_stroke(art_rect, egui::Rounding::same(3.0),
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 210, 190, 60)));
        } else {
            placeholder_art(&painter, art_rect);
        }
    } else {
        placeholder_art(&painter, art_rect);
    }

    ui.add_space(10.0);

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = egui::Vec2::new(0.0, 2.0);
        if let Some(track) = app.current_track() {
            let title  = track.title.clone();
            let artist = track.artist.clone();
            let max_t  = 22usize;
            ui.label(egui::RichText::new(truncate(&title, max_t)).size(DOCK_TITLE).color(TEXT_PRIMARY).strong());
            ui.label(egui::RichText::new(truncate(&artist, max_t + 4)).size(DOCK_ARTIST).color(TEXT_SECONDARY));
        } else {
            ui.label(egui::RichText::new("no track").size(DOCK_TITLE).color(TEXT_SECONDARY));
            ui.label(egui::RichText::new("—").size(DOCK_ARTIST).color(TEXT_DEAD));
        }
    });
}

fn placeholder_art(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, egui::Rounding::same(4.0), panel_glass());
    painter.rect_stroke(rect, egui::Rounding::same(4.0),
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 210, 190, 30)));
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, "♪",
        egui::FontId::proportional(18.0), ACCENT_DIM);
}

// ─── Transport widgets ────────────────────────────────────────────────────────

fn transport_icon(ui: &mut egui::Ui, glyph: &str, active: bool, tip: &str) -> egui::Response {
    let color = if active { ACCENT } else { TEXT_SECONDARY };
    let btn = egui::Button::new(egui::RichText::new(glyph).size(ICON_SIZE).color(color))
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
        .frame(false)
        .min_size(egui::Vec2::new(26.0, 26.0));
    let r = ui.add(btn);
    if active {
        let dot_y = r.rect.bottom() + 2.0;
        let dot_x = r.rect.center().x;
        ui.painter().circle_filled(egui::Pos2::new(dot_x, dot_y), 2.0, ACCENT);
    }
    r.on_hover_text(tip)
}

fn play_button(ui: &mut egui::Ui, playing: bool) -> egui::Response {
    let glyph = if playing { "⏸" } else { "▶" };
    let btn = egui::Button::new(egui::RichText::new(glyph).size(19.0).color(play_btn_icon()))
        .fill(play_btn_fill())
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(PLAY_BTN / 2.0))
        .min_size(egui::Vec2::splat(PLAY_BTN));
    ui.add(btn).on_hover_text(if playing { "Pause (Space)" } else { "Play (Space)" })
}

fn scrubber(ui: &mut egui::Ui, app: &mut MusiqApp, width: f32) {
    let (elapsed, total) = app.engine.position();
    let progress = if total > 0.0 { (elapsed / total).clamp(0.0, 1.0) } else { 0.0 };

    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(width, SCRUBBER_HIT),
        egui::Sense::click_and_drag(),
    );

    let hovered  = response.hovered() || response.dragged();
    app.scrubber_hovered = hovered;

    let vis_h    = if hovered { SCRUBBER_VIS_HOVER } else { SCRUBBER_VIS };
    let vis_y    = rect.center().y - vis_h / 2.0;
    let vis_rect = egui::Rect::from_min_size(
        egui::Pos2::new(rect.left(), vis_y),
        egui::Vec2::new(width, vis_h),
    );
    let painter = ui.painter_at(rect);

    // Track background
    painter.rect_filled(vis_rect, egui::Rounding::same(vis_h / 2.0), scrubber_track());

    // Filled portion — accent colour
    let progress_w = width * progress as f32;
    let prog_rect  = egui::Rect::from_min_size(vis_rect.min, egui::Vec2::new(progress_w, vis_h));
    painter.rect_filled(prog_rect, egui::Rounding::same(vis_h / 2.0), ACCENT);

    if hovered {
        let thumb_x = rect.left() + progress_w;
        // Glow shadow
        painter.circle_filled(egui::Pos2::new(thumb_x, rect.center().y), 7.0,
            egui::Color32::from_rgba_unmultiplied(0, 210, 190, 30));
        painter.circle_filled(egui::Pos2::new(thumb_x, rect.center().y), 5.0, ACCENT);
    }

    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let ratio = ((pos.x - rect.left()) / width).clamp(0.0, 1.0) as f64;
            app.seek_to(ratio);
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn submit_welcome_path(app: &mut MusiqApp) {
    let trimmed = app.input.trim().to_string();
    if !trimmed.is_empty() {
        app.scan_path(std::path::PathBuf::from(trimmed));
        app.input.clear();
    }
}

fn fmt_time(secs: f64) -> String {
    let s = secs as u64;
    format!("{:02}:{:02}", s / 60, s % 60)
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars { s.to_string() }
    else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn truncate_path(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars { s.to_string() }
    else {
        let n    = max_chars.saturating_sub(1);
        let skip = s.chars().count().saturating_sub(n);
        let mut out = String::from("…");
        out.extend(s.chars().skip(skip));
        out
    }
}
