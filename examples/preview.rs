//! Visual preview of the GNOME-style banner card.
//!
//! `cargo run --release --example preview`

#[path = "../src/banner.rs"]
mod banner;

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::widget::Column;
use cosmic::iced::{Alignment, Length, Size};
use cosmic::widget::{container, scrollable};
use cosmic::{Element, executor};
use cosmic_notifications_util::{ActionId, Hint, Image, Notification};
use std::time::SystemTime;

fn main() -> cosmic::iced::Result {
    let settings = Settings::default().size(Size::new(480.0, 720.0));
    cosmic::app::run::<Preview>(settings, ())
}

#[derive(Debug, Clone)]
enum Message {
    Activate(u32),
    Dismiss(u32),
    Action(u32, ActionId),
}

struct Preview {
    core: Core,
    notifications: Vec<Notification>,
}

#[allow(clippy::too_many_arguments)]
fn sample(
    id: u32,
    app_name: &str,
    app_icon: &str,
    image: Option<&str>,
    summary: &str,
    body: &str,
    actions: &[(&str, &str)],
    urgency: u8,
    minutes_ago: u64,
) -> Notification {
    let mut hints = vec![Hint::Urgency(urgency)];
    if let Some(name) = image {
        hints.push(Hint::Image(Image::Name(name.into())));
    }
    Notification {
        id,
        app_name: app_name.into(),
        app_icon: app_icon.into(),
        summary: summary.into(),
        body: body.into(),
        actions: actions
            .iter()
            .map(|(id, label)| (id.parse().unwrap(), label.to_string()))
            .collect(),
        hints,
        expire_timeout: -1,
        time: SystemTime::now() - std::time::Duration::from_secs(minutes_ago * 60),
    }
}

/// Synthetic 96x96 RGBA sender picture (procedural, no external asset):
/// a muted green ground with a lighter head-and-shoulders silhouette.
fn avatar_raster() -> Image {
    let (w, h) = (96u32, 96u32);
    let mut data = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (fx, fy) = (x as f32 - 48.0, y as f32);
            let head = fx * fx + (fy - 38.0) * (fy - 38.0) < 18.0 * 18.0;
            let body = fx * fx / (34.0 * 34.0) + (fy - 98.0) * (fy - 98.0) / (36.0 * 36.0) < 1.0;
            let (r, g, b) = if head || body { (0xd9, 0xe4, 0xd2) } else { (0x4f, 0x6f, 0x52) };
            data.extend_from_slice(&[r, g, b, 0xff]);
        }
    }
    Image::Data { width: w, height: h, data }
}

impl cosmic::Application for Preview {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "io.github.agustine_leo.CosmicExtNotificationsPreview";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _: ()) -> (Self, Task<Message>) {
        let notifications = vec![
            sample(
                1, "Whatsie", "whatsie", None, "Mum",
                "Dinner at 7? Bring the dish you borrowed last week if you still \
                 have it, and tell your brother too.\n\nThis line should be cut off.",
                &[("reply", "Reply"), ("read", "Mark as Read")], 1, 0,
            ),
            sample(
                2, "Zen Browser", "zen", None, "Standup in 5 minutes",
                "meet.google.com · Platform team", &[("default", "Default")], 1, 137,
            ),
            sample(
                3, "Battery", "battery-caution-symbolic", None, "Battery at 6%",
                "Plug in now. The laptop suspends in about 4 minutes.", &[], 2, 251,
            ),
            sample(
                4, "Files", "folder-download", None, "Download complete",
                "fedora-workstation-45.iso", &[("open", "Open"), ("folder", "Show in Folder")], 0, 488,
            ),
            sample(
                5, "", "", None, "A notification without any icon or app name",
                "Falls back to dialog-information-symbolic.", &[], 1, 612,
            ),
        ];
        let mut notifications = notifications;
        notifications[0].hints.push(Hint::Image(avatar_raster()));
        (
            Preview {
                core,
                notifications,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Activate(id) => println!("activate {id}"),
            Message::Dismiss(id) => println!("dismiss {id}"),
            Message::Action(id, action) => println!("action {id}: {action}"),
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let cards = self.notifications.iter().map(|n| {
            banner::card(
                n,
                // Samples only use named icons, so the scale (which only
                // sizes image-data rasters) does not matter here.
                2.0,
                Message::Activate(n.id),
                Message::Dismiss(n.id),
                move |action| Message::Action(n.id, action),
            )
        });
        let column = Column::with_children(cards)
            .spacing(8)
            .padding(16)
            .width(Length::Fill)
            .align_x(Alignment::Center);
        scrollable(container(column).width(Length::Fill)).into()
    }
}
