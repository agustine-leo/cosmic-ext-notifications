use cosmic::Element;
use cosmic::iced::{Alignment, Background, Color, Length};
use cosmic::iced::widget::{Column, Row};
use cosmic::iced::core::text::{Ellipsize, EllipsizeHeightLimit};
use cosmic::widget::{button, container, divider, icon, mouse_area, text};
use cosmic_notifications_util::{ActionId, Notification};

/// 16x16 close glyph, embedded because some icon themes blank
/// `window-close-symbolic`. Rendered symbolic so it takes the theme colour.
const CLOSE_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16"><path fill="#232323" d="M4.3 4.3a1 1 0 0 1 1.4 0L8 6.6l2.3-2.3a1 1 0 1 1 1.4 1.4L9.4 8l2.3 2.3a1 1 0 0 1-1.4 1.4L8 9.4l-2.3 2.3a1 1 0 0 1-1.4-1.4L6.6 8 4.3 5.7a1 1 0 0 1 0-1.4z"/></svg>"##;

const PADDING: u16 = 12;
const ICON_SIZE: u16 = 40;
const ACTION_HEIGHT: f32 = 40.0;

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

/// Corner radius of the banner card (theme `radius_m`).
pub fn card_radius(theme: &cosmic::Theme) -> [f32; 4] {
    theme.cosmic().corner_radii.radius_m
}

/// GNOME-style notification banner: icon, bold title, dim body, × at the
/// top right, optional equal-width action row flush to the bottom.
pub fn card<'a, M: Clone + 'a>(
    n: &'a Notification,
    on_activate: M,
    on_dismiss: M,
    on_action: impl Fn(ActionId) -> M + 'a,
) -> Element<'a, M> {
    let notif_icon = n
        .notification_icon()
        .unwrap_or_else(|| icon::from_name("dialog-information-symbolic").icon())
        .size(ICON_SIZE);

    let title = text::heading(n.summary.lines().next().unwrap_or_default())
        .width(Length::Fill)
        .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(1)));

    let mut texts = Column::new().push(title).spacing(2).width(Length::Fill);
    if let Some(body) = body_preview(&n.body) {
        texts = texts.push(
            text::body(body)
                .width(Length::Fill)
                .ellipsize(Ellipsize::End(EllipsizeHeightLimit::Lines(2)))
                .class(cosmic::theme::Text::Custom(|theme| {
                    let mut c: Color = theme.cosmic().background(theme.transparent).component.on.into();
                    c.a *= 0.7;
                    cosmic::iced::widget::text::Style {
                        color: Some(c),
                        ..Default::default()
                    }
                })),
        );
    }

    let close = button::custom(
        icon::icon(icon::from_svg_bytes(CLOSE_SVG).symbolic(true)).size(16),
    )
    .padding(4)
    .on_press(on_dismiss)
    .class(cosmic::theme::Button::Icon);

    // Only icon + text activate; × is a sibling outside the mouse_area.
    let activate = mouse_area(
        Row::new()
            .push(notif_icon)
            .push(texts)
            .spacing(12)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .on_press(on_activate);

    let upper = Row::new()
        .push(activate)
        .push(close)
        .spacing(8)
        .align_y(Alignment::Start);

    let upper = container(upper).padding(PADDING).width(Length::Fill);

    let mut content = Column::new().push(upper).width(Length::Fill);

    let actions = action_buttons(&n.actions);
    if !actions.is_empty() {
        let count = actions.len();
        let mut row = Row::new()
            .height(Length::Fixed(ACTION_HEIGHT))
            .width(Length::Fill);
        for (i, (id, label)) in actions.into_iter().enumerate() {
            if i > 0 {
                row = row.push(divider::vertical::default());
            }
            let msg = on_action(id);
            row = row.push(
                button::custom(
                    container(text::body(label).ellipsize(Ellipsize::End(
                        EllipsizeHeightLimit::Lines(1),
                    )))
                    .center(Length::Fill)
                    .padding([0, 8]),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .on_press(msg)
                .class(action_class(i == 0, i + 1 == count)),
            );
        }
        content = content
            .push(divider::horizontal::default())
            .push(row);
    }

    container(content)
        .width(Length::Fixed(BANNER_WIDTH))
        .class(cosmic::theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            let component = &cosmic.background(theme.transparent).component;
            cosmic::iced::widget::container::Style {
                icon_color: Some(component.on.into()),
                text_color: Some(component.on.into()),
                background: Some(Background::Color(component.base.into())),
                border: cosmic::iced::Border {
                    radius: card_radius(theme).into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

/// Flat action button; only the outer bottom corners follow the card radius.
fn action_class(first: bool, last: bool) -> cosmic::theme::Button {
    fn style(first: bool, last: bool, theme: &cosmic::Theme, bg: Option<Color>) -> button::Style {
        let r = card_radius(theme);
        let on: Color = theme.cosmic().background(theme.transparent).component.on.into();
        let mut s = button::Style::new();
        s.background = bg.map(Background::Color);
        // iced radius order: top-left, top-right, bottom-right, bottom-left
        s.border_radius = [0.0, 0.0, if last { r[2] } else { 0.0 }, if first { r[3] } else { 0.0 }].into();
        s.text_color = Some(on);
        s.icon_color = Some(on);
        s
    }
    cosmic::theme::Button::Custom {
        active: Box::new(move |_, t| style(first, last, t, None)),
        disabled: Box::new(move |t| style(first, last, t, None)),
        hovered: Box::new(move |_, t| {
            style(first, last, t, Some(t.cosmic().background(t.transparent).component.hover.into()))
        }),
        pressed: Box::new(move |_, t| {
            style(first, last, t, Some(t.cosmic().background(t.transparent).component.pressed.into()))
        }),
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
        let smallest_card = f32::from(2 * PADDING + ICON_SIZE);
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
}
