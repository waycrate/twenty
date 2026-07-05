use iced::widget::{column, text};
use iced::{Alignment, Command, Element, Event, Length, Subscription, Theme, event, time};
use std::time::Duration;

use iced_sessionlock::{MultiApplication, actions::UnLockAction, settings::Settings};

pub fn lock(dark: bool, secs: u64) -> Result<(), iced_sessionlock::Error> {
    let settings = Settings {
        flags: (dark, secs),
        ..Settings::default()
    };
    Counter::run(settings)
}

struct Counter {
    value: u64,
    dark: bool,
}

#[derive(Debug, Clone)]
enum Message {
    DecrementCounter,
    IcedEvent(Event),
    Unlock,
}

impl MultiApplication for Counter {
    type Message = Message;
    type Flags = (bool, u64);
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let (dark, secs) = flags;
        (Self { value: secs, dark }, Command::none())
    }

    fn namespace(&self) -> String {
        String::from("Twenty - a 20-20-20 rule enforcer")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch(vec![
            event::listen().map(Message::IcedEvent),
            time::every(Duration::from_secs(1)).map(|_| Message::DecrementCounter),
        ])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(_event) => Command::none(),
            Message::DecrementCounter => {
                if self.value > 0 {
                    self.value -= 1;
                    Command::none()
                } else {
                    Command::perform(async {}, |_| Message::Unlock)
                }
            }
            Message::Unlock => Command::single(UnLockAction.into()),
        }
    }

    fn view(&self, _id: iced::window::Id) -> Element<'_, Message> {
        column![
            text(format!("{}s left", self.value)).size(100),
            text("Look away.").size(100)
        ]
        .padding(200)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn theme(&self) -> Theme {
        if self.dark { Theme::Dark } else { Theme::Light }
    }
}
