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
    let settings = Settings::default().size(Size::new(520.0, 900.0));
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

fn sample(
    id: u32,
    app_icon: &str,
    image: Option<&str>,
    summary: &str,
    body: &str,
    actions: &[(&str, &str)],
) -> Notification {
    Notification {
        id,
        app_name: "Preview".into(),
        app_icon: app_icon.into(),
        summary: summary.into(),
        body: body.into(),
        actions: actions
            .iter()
            .map(|(id, label)| (id.parse().unwrap(), label.to_string()))
            .collect(),
        hints: image
            .map(|name| vec![Hint::Image(Image::Name(name.into()))])
            .unwrap_or_default(),
        expire_timeout: -1,
        time: SystemTime::now(),
    }
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
            sample(1, "preferences-system-time", None, "Alarm", "12:30", &[]),
            sample(
                2,
                "mail-unread",
                None,
                "10 New Emails",
                "New emails from <b>Angela Avery</b>, Jan Hendricks, Roger Law, \
                 Lucy Goffman, Anne Ball, Ivan Rose, James Brooks, Jane Featherstone, \
                 Frank Colebrook and Hannah Hill.\n\nThis line should be cut off.",
                &[],
            ),
            sample(
                3,
                "web-browser",
                None,
                "Download complete",
                "fedora-workstation.iso finished downloading.",
                &[("default", "Default"), ("open", "Open")],
            ),
            sample(
                4,
                "",
                Some("dialog-warning"),
                "Low Disk Space",
                "Only 30 MB remaining.",
                &[
                    ("snooze", "Snooze"),
                    ("stop", "Stop"),
                    ("examine", "Examine Disk"),
                ],
            ),
            sample(
                5,
                "",
                None,
                "A notification without any icon",
                "Falls back to dialog-information-symbolic.",
                &[],
            ),
        ];
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
                1.0,
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
