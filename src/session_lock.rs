use iced::widget::{column, text};
use iced::{event, time, Alignment, Command, Element, Event, Length, Subscription, Theme};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use iced_sessionlock::actions::UnLockAction;
use iced_sessionlock::settings::Settings;
use iced_sessionlock::MultiApplication;

pub fn lock() -> Result<(), iced_sessionlock::Error> {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone)]
enum Message {
    DecrementCounter,
    IcedEvent(Event),
    Unlock,
}

pub static UNLOCKED: AtomicBool = AtomicBool::new(false);
pub static IS_DARKMODE: AtomicBool = AtomicBool::new(true);

impl MultiApplication for Counter {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self { value: 20 }, Command::none())
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
            Message::Unlock => {
                UNLOCKED.store(true, Ordering::Relaxed);
                Command::single(UnLockAction.into())
            }
        }
    }

    fn view(&self, _id: iced::window::Id) -> Element<Message> {
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
        if IS_DARKMODE.load(Ordering::Relaxed) {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
