use cosmic_notifications_util::ActionId;

#[allow(dead_code)]
pub const BANNER_WIDTH: f32 = 420.0;

/// Plain-text one-line preview of a notification body: tags stripped,
/// entities decoded, whitespace collapsed. `None` if nothing remains.
#[allow(dead_code)]
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
                i += end + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    let decoded = out
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}

/// Buttons to show on a banner: never `Default`, order kept, max 3.
#[allow(dead_code)]
pub fn action_buttons(actions: &[(ActionId, String)]) -> Vec<(ActionId, String)> {
    actions
        .iter()
        .filter(|(id, _)| !matches!(id, ActionId::Default))
        .take(3)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn body_preview_strips_markup_and_collapses_whitespace() {
        assert_eq!(body_preview("<b>Disk</b>   is\n\n  <a href=\"x\">full</a>").as_deref(), Some("Disk is full"));
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
