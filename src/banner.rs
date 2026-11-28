use cosmic::Element;
use cosmic::iced::{Alignment, Background, Color, Length};
use cosmic::iced::widget::{Column, Row, Space};
use cosmic::iced::core::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::widget::{button, container, icon, mouse_area, text};
use cosmic_notifications_util::{ActionId, Image, Notification};
use std::time::SystemTime;

/// 16x16 close glyph, embedded because some icon themes blank
/// `window-close-symbolic`. Rendered symbolic so it takes the theme colour.
const CLOSE_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="#232323" d="M4.3 4.3a1 1 0 0 1 1.4 0L8 6.6l2.3-2.3a1 1 0 1 1 1.4 1.4L9.4 8l2.3 2.3a1 1 0 0 1-1.4 1.4L8 9.4l-2.3 2.3a1 1 0 0 1-1.4-1.4L6.6 8 4.3 5.7a1 1 0 0 1 0-1.4z"/></svg>"##;

const PADDING: f32 = 12.0;
const ICON_SIZE: u16 = 32;
/// Width of the time column; fixed so every banner's rule lines up.
const TIME_COL: f32 = 72.0;
const RADIUS: f32 = 6.0;

// Timetable palette: navy strip, paper type, slate secondary, one red.
const NAVY: Color = Color::from_rgb8(0x12, 0x1c, 0x28);
const PAPER: Color = Color::from_rgb8(0xf2, 0xf0, 0xea);
const PAPER_DIM: Color = Color::from_rgba8(0xf2, 0xf0, 0xea, 0.74);
const SLATE: Color = Color::from_rgb8(0x8c, 0x98, 0xa6);
const RED: Color = Color::from_rgb8(0xc8, 0x24, 0x3a);
const RULE: Color = Color::from_rgb8(0x2a, 0x36, 0x47);
const HOVER: Color = Color::from_rgb8(0x2a, 0x36, 0x47);
const PRESSED: Color = Color::from_rgb8(0x36, 0x45, 0x5a);

pub const BANNER_WIDTH: f32 = 420.0;
/// Minimum height of a banner's layer surface. Must stay below the smallest
/// card, or every banner is padded and stacked banners drift apart.
pub const SURFACE_MIN_HEIGHT: f32 = 1.0;

/// Plain-text one-line preview of a notification body: tags stripped,
/// entities decoded, whitespace collapsed. `None` if nothing remains.
pub fn body_preview(body: &str) -> Option<String> {
    let mut out = String::with_capacity(body.len());
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '<'
            && chars
                .get(i + 1)
                .is_some_and(|n| n.is_ascii_alphabetic() || *n == '/')
        {
            if let Some(end) = chars[i..].iter().position(|&x| x == '>') {
                if is_block_tag(&chars[i + 1..i + end]) {
                    out.push(' ');
                }
                i += end + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}

/// Whether a tag (the text between `<` and `>`) breaks a line or block, so
/// removing it must leave a space; inline tags (b, i, a, span, ...) do not.
fn is_block_tag(inner: &[char]) -> bool {
    let name: String = inner
        .iter()
        .skip_while(|c| **c == '/')
        .take_while(|c| !c.is_whitespace() && **c != '/' && **c != '>')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    matches!(
        name.as_str(),
        "br" | "p" | "div" | "li" | "ul" | "ol" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}

/// Buttons to show on a banner: never `Default`, order kept, max 3.
pub fn action_buttons(actions: &[(ActionId, String)]) -> Vec<(ActionId, String)> {
    actions
        .iter()
        .filter(|(id, _)| !matches!(id, ActionId::Default))
        .take(3)
        .cloned()
        .collect()
}

/// Which icon a banner draws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconChoice {
    /// `Notification::notification_icon()` at `ICON_SIZE` (file/name images,
    /// or no image at all).
    Default,
    /// The raw `image-data` raster the app sent, at this logical size
    /// (≤ `ICON_SIZE`), centred in the icon slot.
    Raster(f32),
}

/// Pick the banner icon for `n` on a surface with output scale `scale`.
/// The picture the app sent always wins; it is only ever sized down.
pub fn icon_choice(n: &Notification, scale: f32) -> IconChoice {
    match n.image() {
        Some(Image::Data { width, height, .. }) => {
            IconChoice::Raster(raster_display_size(*width, *height, scale))
        }
        _ => IconChoice::Default,
    }
}

/// Logical size for a `w`x`h` raster at output scale `scale`:
/// `min(ICON_SIZE, w / scale, h / scale)`, so it is never drawn with more
/// physical pixels than it has. A non-positive or non-finite scale counts
/// as 1; the result is at least 1.
pub fn raster_display_size(w: u32, h: u32, scale: f32) -> f32 {
    let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    (w.min(h) as f32 / scale).clamp(1.0, f32::from(ICON_SIZE))
}

/// Width of one figure cell, and of the colon cell, for 20px bold digits.
const DIGIT_CELL: f32 = 12.0;
const COLON_CELL: f32 = 6.0;

/// `HH:MM` set in fixed cells, so every banner's figures and colon line up
/// like a timetable column whatever the digits (the font's own figures are
/// proportional and iced cannot request tabular ones).
fn time_figures<'a, M: 'a>(label: &str, color: Color) -> Element<'a, M> {
    let mut row = Row::new();
    for c in label.chars() {
        let cell = if c == ':' { COLON_CELL } else { DIGIT_CELL };
        row = row.push(
            container(
                text(c.to_string())
                    .size(20)
                    .font(cosmic::font::bold())
                    .class(cosmic::theme::Text::Color(color)),
            )
            .center_x(Length::Fixed(cell)),
        );
    }
    row.into()
}

/// Put a zero-width space after every space. With end-ellipsizing, the
/// text engine otherwise breaks before a space and starts the next line
/// with it, breaking the column's left edge.
pub fn wrap_safe(s: &str) -> String {
    s.replace(' ', " \u{200B}")
}

/// The icon-theme name a banner would draw, if it draws one by name
/// (`image-path`/`image` hint as a name, or `app_icon` that is not a file).
pub fn icon_name(n: &Notification) -> Option<&str> {
    match n.image() {
        Some(Image::Name(name)) => Some(name.as_str()),
        Some(_) => None,
        None => {
            let a = n.app_icon.as_str();
            (!a.is_empty() && !a.starts_with("file://") && !a.starts_with('/')).then_some(a)
        }
    }
}

/// Corner radius of the banner card. Fixed by the timetable world, not the
/// theme, so the compositor clip always matches the drawn strip.
pub fn card_radius(_theme: &cosmic::Theme) -> [f32; 4] {
    [RADIUS; 4]
}

/// `HH:MM`, 24-hour, as a timetable prints it.
pub fn hhmm(hour: u32, minute: u32) -> String {
    format!("{:02}:{:02}", hour % 24, minute % 60)
}

/// Local wall-clock time of `t` as `HH:MM`; empty if it cannot be resolved.
pub fn clock_label(t: SystemTime) -> String {
    let Ok(secs) = t.duration_since(SystemTime::UNIX_EPOCH) else {
        return String::new();
    };
    let Ok(secs) = libc::time_t::try_from(secs.as_secs()) else {
        return String::new();
    };
    // SAFETY: localtime_r writes only into `tm`, which we own.
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&secs, &mut tm) }.is_null() {
        return String::new();
    }
    hhmm(tm.tm_hour as u32, tm.tm_min as u32)
}

/// App name for the time column, or `None` when it would only repeat the
/// title (nothing is labelled twice) or is empty.
pub fn app_label<'a>(app_name: &'a str, title: &str) -> Option<&'a str> {
    let name = app_name.trim();
    (!name.is_empty() && !name.eq_ignore_ascii_case(title.trim())).then_some(name)
}

/// Urgency tier, as the time column prints it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tier {
    Low,
    Normal,
    Critical,
}

pub fn tier(urgency: u8) -> Tier {
    match urgency {
        0 => Tier::Low,
        2 => Tier::Critical,
        _ => Tier::Normal,
    }
}

/// Timetable banner: a fixed-width time column (time, app) ruled off from
/// the content (picture, sender, message, ×, actions). The time column sits
/// at the same x on every banner, so a stack reads as one table. `scale` is
/// the output scale factor, used to keep image-data icons sharp.
pub fn card<'a, M: Clone + 'a>(
    n: &'a Notification,
    scale: f32,
    on_activate: M,
    on_dismiss: M,
    on_action: impl Fn(ActionId) -> M + 'a,
) -> Element<'a, M> {
    let tier = tier(n.urgency());
    let summary = n.summary.lines().next().unwrap_or_default();

    // ---- time column ----
    let figure_color = match tier {
        Tier::Low => SLATE,
        _ => PAPER,
    };
    let mut time_col = Column::new()
        .push(time_figures(&clock_label(n.time), figure_color))
        .spacing(2)
        .width(Length::Fill);
    if let Some(app) = app_label(&n.app_name, summary) {
        time_col = time_col.push(
            text(app)
                .size(11)
                .width(Length::Fill)
                .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
                .class(cosmic::theme::Text::Color(match tier {
                    // Full paper on red: dimmed paper falls under 4.5:1.
                    Tier::Critical => PAPER,
                    _ => SLATE,
                })),
        );
    }
    let time_col = container(time_col)
        .padding([PADDING, 0.0, PADDING, PADDING])
        .width(Length::Fixed(TIME_COL))
        .height(Length::Fill)
        .class(cosmic::theme::Container::custom(move |_| {
            cosmic::iced::widget::container::Style {
                background: (tier == Tier::Critical).then_some(Background::Color(RED)),
                border: cosmic::iced::Border {
                    radius: [RADIUS, 0.0, 0.0, RADIUS].into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }));

    let rule = container(Space::new().width(Length::Fixed(1.0)).height(Length::Fill))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .class(cosmic::theme::Container::custom(move |_| {
            cosmic::iced::widget::container::Style {
                background: (tier != Tier::Critical).then_some(Background::Color(RULE)),
                ..Default::default()
            }
        }));

    // ---- content column ----
    let fallback = || icon::from_name("dialog-information-symbolic").icon();
    let notif_icon: Element<'a, M> = match icon_choice(n, scale) {
        IconChoice::Default => match icon_name(n) {
            // A name the icon theme cannot resolve would draw an empty slot.
            Some(name) if icon::from_name(name).path().is_none() => fallback(),
            _ => n.notification_icon().unwrap_or_else(fallback),
        }
        .size(ICON_SIZE)
        .into(),
        IconChoice::Raster(size) => {
            let raster = n
                .notification_icon()
                .unwrap_or_else(fallback)
                .width(Length::Fixed(size))
                .height(Length::Fixed(size))
                .content_fit(cosmic::iced::ContentFit::Contain);
            container(raster)
                .center(Length::Fixed(f32::from(ICON_SIZE)))
                .into()
        }
    };

    let title = text(summary)
        .size(15)
        .font(cosmic::font::bold())
        .width(Length::Fill)
        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)))
        .class(cosmic::theme::Text::Color(PAPER));

    let mut texts = Column::new().push(title).spacing(2).width(Length::Fill);
    if let Some(body) = body_preview(&n.body) {
        texts = texts.push(
            text(wrap_safe(&body))
                .size(14)
                .width(Length::Fill)
                .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(2)))
                .class(cosmic::theme::Text::Color(PAPER_DIM)),
        );
    }

    let close = button::custom(
        icon::icon(icon::from_svg_bytes(CLOSE_SVG).symbolic(true)).size(16),
    )
    .padding(4)
    .on_press(on_dismiss)
    .class(flat_button(SLATE));

    // Only picture + text activate; × and actions sit outside the mouse_area.
    let activate = mouse_area(
        Row::new()
            .push(notif_icon)
            .push(texts)
            .spacing(10)
            .align_y(Alignment::Start)
            .width(Length::Fill),
    )
    .on_press(on_activate);

    let mut content = Column::new()
        .push(
            Row::new()
                .push(activate)
                .push(close)
                .spacing(4)
                .align_y(Alignment::Start),
        )
        .spacing(6)
        .width(Length::Fill);

    let actions = action_buttons(&n.actions);
    if !actions.is_empty() {
        let mut row = Row::new().spacing(4);
        for (id, label) in actions {
            row = row.push(
                button::custom(
                    text(label)
                        .size(14)
                        .font(cosmic::font::bold())
                        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1))),
                )
                .padding([4, 8])
                .on_press(on_action(id))
                .class(flat_button(PAPER)),
            );
        }
        // Pull the first label's text in line with the title above it.
        content = content.push(
            container(row).padding([0.0, 0.0, 0.0, f32::from(ICON_SIZE) + 10.0 - 8.0]),
        );
    }

    let content = container(content)
        .padding([PADDING, PADDING - 4.0, PADDING, PADDING])
        .width(Length::Fill);

    container(
        Row::new()
            .push(time_col)
            .push(rule)
            .push(content)
            .height(Length::Shrink),
    )
    .width(Length::Fixed(BANNER_WIDTH))
    .class(cosmic::theme::Container::custom(|_| {
        cosmic::iced::widget::container::Style {
            icon_color: Some(PAPER),
            text_color: Some(PAPER),
            background: Some(Background::Color(NAVY)),
            border: cosmic::iced::Border {
                radius: RADIUS.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }))
    .into()
}

/// Flat text/icon button on navy: no fill at rest, a lifted navy on hover,
/// deeper on press. `fg` colours its label or icon.
fn flat_button(fg: Color) -> cosmic::theme::Button {
    fn style(fg: Color, bg: Option<Color>) -> button::Style {
        let mut s = button::Style::new();
        s.background = bg.map(Background::Color);
        s.border_radius = 4.0.into();
        s.text_color = Some(fg);
        s.icon_color = Some(fg);
        s
    }
    cosmic::theme::Button::Custom {
        active: Box::new(move |_, _| style(fg, None)),
        disabled: Box::new(move |_| style(SLATE, None)),
        hovered: Box::new(move |_, _| style(PAPER, Some(HOVER))),
        pressed: Box::new(move |_, _| style(PAPER, Some(PRESSED))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn body_preview_strips_markup_and_collapses_whitespace() {
        assert_eq!(body_preview("<b>Disk</b>   is\n\n  <a href=\"x\">full</a>").as_deref(), Some("Disk is full"));
    }
    #[test] fn body_preview_tag_separates_words() {
        assert_eq!(body_preview("a<br>b").as_deref(), Some("a b"));
        assert_eq!(body_preview("<p>one</p><p>two</p>").as_deref(), Some("one two"));
    }
    #[test] fn body_preview_decodes_nbsp_to_space() {
        assert_eq!(body_preview("a&nbsp;b").as_deref(), Some("a b"));
    }
    #[test] fn surface_min_height_does_not_pad_smallest_card() {
        let smallest_card = 2.0 * PADDING + f32::from(ICON_SIZE);
        assert!(SURFACE_MIN_HEIGHT < smallest_card, "{SURFACE_MIN_HEIGHT} >= {smallest_card}");
    }
    #[test] fn body_preview_inline_tags_do_not_split_words() {
        assert_eq!(body_preview("<b>Dis</b>k").as_deref(), Some("Disk"));
        assert_eq!(body_preview("<b>Name</b>: hi").as_deref(), Some("Name: hi"));
        assert_eq!(body_preview("<a href=\"x\">link</a>.").as_deref(), Some("link."));
        assert_eq!(body_preview("a<br/>b").as_deref(), Some("a b"));
        assert_eq!(body_preview("<p>one</p><p>two</p>").as_deref(), Some("one two"));
    }
    #[test] fn body_preview_empty_is_none() { assert_eq!(body_preview("  \n "), None); }
    #[test] fn action_buttons_drops_default_and_keeps_order() {
        let a = vec![(ActionId::Default, "Open".into()), (ActionId::Custom("snooze".into()), "Snooze".into()), (ActionId::Custom("stop".into()), "Stop".into())];
        assert_eq!(action_buttons(&a).iter().map(|(_, l)| l.as_str()).collect::<Vec<_>>(), ["Snooze", "Stop"]);
    }
    #[test] fn action_buttons_caps_at_three() {
        let a: Vec<_> = (0..5).map(|i| (ActionId::Custom(i.to_string()), format!("A{i}"))).collect();
        assert_eq!(action_buttons(&a).len(), 3);
    }
    #[test] fn action_buttons_default_only_is_empty() {
        assert!(action_buttons(&[(ActionId::Default, "Open".into())]).is_empty());
    }
    #[test] fn action_buttons_single_custom_is_shown() {
        assert_eq!(action_buttons(&[(ActionId::Custom("open".into()), "Open".into())]).len(), 1);
    }
    fn data(w: u32, h: u32) -> Image { Image::Data { width: w, height: h, data: vec![0; (w * h * 4) as usize] } }
    fn notif(app_icon: &str, image: Option<Image>) -> Notification {
        Notification {
            id: 1,
            app_name: "app".into(),
            app_icon: app_icon.into(),
            summary: "s".into(),
            body: String::new(),
            actions: Vec::new(),
            hints: image.map(|i| vec![cosmic_notifications_util::Hint::Image(i)]).unwrap_or_default(),
            expire_timeout: -1,
            time: std::time::SystemTime::UNIX_EPOCH,
        }
    }
    #[test] fn raster_is_sized_by_output_scale() {
        assert_eq!(icon_choice(&notif("", Some(data(48, 48))), 2.0), IconChoice::Raster(24.0));
        assert_eq!(icon_choice(&notif("", Some(data(48, 48))), 1.0), IconChoice::Raster(32.0));
        assert_eq!(icon_choice(&notif("", Some(data(160, 160))), 2.0), IconChoice::Raster(32.0));
        assert_eq!(icon_choice(&notif("", Some(data(48, 48))), 1.5), IconChoice::Raster(32.0));
    }
    #[test] fn app_icon_never_replaces_the_sent_picture() {
        assert_eq!(icon_choice(&notif("org.telegram.desktop", Some(data(48, 48))), 2.0), IconChoice::Raster(24.0));
        assert_eq!(icon_choice(&notif("telegram", Some(data(64, 64))), 2.0), IconChoice::Raster(32.0));
    }
    #[test] fn file_name_and_missing_images_use_default() {
        assert_eq!(icon_choice(&notif("telegram", Some(Image::Name("avatar".into()))), 2.0), IconChoice::Default);
        assert_eq!(icon_choice(&notif("telegram", Some(Image::File("/tmp/a.png".into()))), 2.0), IconChoice::Default);
        assert_eq!(icon_choice(&notif("telegram", None), 2.0), IconChoice::Default);
        assert_eq!(icon_choice(&notif("", None), 1.0), IconChoice::Default);
    }
    #[test] fn raster_display_size_is_native_over_scale_capped_at_icon_size() {
        assert_eq!(raster_display_size(48, 48, 2.0), 24.0);
        assert_eq!(raster_display_size(48, 48, 1.0), f32::from(ICON_SIZE));
        assert_eq!(raster_display_size(160, 160, 2.0), f32::from(ICON_SIZE));
        assert_eq!(raster_display_size(64, 32, 2.0), 16.0, "smaller side limits");
        assert_eq!(raster_display_size(512, 512, 1.0), f32::from(ICON_SIZE));
    }
    #[test] fn raster_display_size_guards_bad_input() {
        assert_eq!(raster_display_size(0, 0, 2.0), 1.0);
        assert_eq!(raster_display_size(48, 48, 0.0), f32::from(ICON_SIZE));
        assert_eq!(raster_display_size(48, 48, -2.0), f32::from(ICON_SIZE));
        assert_eq!(raster_display_size(48, 48, f32::NAN), f32::from(ICON_SIZE));
    }
    #[test] fn hhmm_pads_and_wraps() {
        assert_eq!(hhmm(9, 5), "09:05");
        assert_eq!(hhmm(23, 59), "23:59");
        assert_eq!(hhmm(24, 60), "00:00");
    }
    #[test] fn clock_label_is_hh_mm() {
        let l = clock_label(SystemTime::now());
        assert_eq!(l.len(), 5, "{l}");
        assert_eq!(&l[2..3], ":");
    }
    #[test] fn app_label_skips_empty_and_repeats() {
        assert_eq!(app_label("WhatsApp", "Mum"), Some("WhatsApp"));
        assert_eq!(app_label("  ", "Mum"), None);
        assert_eq!(app_label("Battery", "battery"), None);
    }
    #[test] fn tier_maps_urgency() {
        assert_eq!(tier(0), Tier::Low);
        assert_eq!(tier(1), Tier::Normal);
        assert_eq!(tier(2), Tier::Critical);
        assert_eq!(tier(7), Tier::Normal);
    }
    #[test] fn icon_name_picks_drawn_name_only() {
        assert_eq!(icon_name(&notif("zen", None)), Some("zen"));
        assert_eq!(icon_name(&notif("", None)), None);
        assert_eq!(icon_name(&notif("file:///a.png", None)), None);
        assert_eq!(icon_name(&notif("zen", Some(Image::Name("avatar".into())))), Some("avatar"));
        assert_eq!(icon_name(&notif("zen", Some(data(8, 8)))), None);
    }
    #[test] fn wrap_safe_breaks_after_spaces_only() {
        assert_eq!(wrap_safe("a b"), "a \u{200B}b");
        assert_eq!(wrap_safe("ab"), "ab");
    }
    #[test] fn time_cells_fit_the_time_column() {
        assert!(PADDING + 4.0 * DIGIT_CELL + COLON_CELL <= TIME_COL - 4.0);
    }
}
