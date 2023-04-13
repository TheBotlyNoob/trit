use iced::widget::container;
use iced::{Application, Command, Element, Length, Settings};

use numeric_input::numeric_input;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

// SAFETY: This application is single threaded, so using AssumeSingleThreaded is allowed.
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

pub fn main() -> iced::Result {
    Component::run(Settings::default())
}

#[derive(Default)]
struct Component {
    value: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    NumericInputChanged(Option<u32>),
}

impl Application for Component {
    type Message = Msg;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = iced::theme::Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Msg>) {
        (
            Self::default(),
            #[cfg(target_arch = "wasm32")]
            {
                use iced_native::{command, window};
                let window = web_sys::window().unwrap();
                let (width, height) = (
                    (window.inner_width().unwrap().as_f64().unwrap()) as u32,
                    (window.inner_height().unwrap().as_f64().unwrap()) as u32,
                );
                Command::single(command::Action::Window(window::Action::Resize {
                    width,
                    height,
                }))
            },
            #[cfg(not(target_arch = "wasm32"))]
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Component - Iced")
    }

    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::NumericInputChanged(value) => {
                self.value = value;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Msg> {
        container(numeric_input(self.value, Msg::NumericInputChanged))
            .padding(20)
            .height(Length::Fill)
            .center_y()
            .into()
    }
}

mod numeric_input {
    use iced::alignment::{self, Alignment};
    use iced::widget::{self, button, row, text, text_input};
    use iced::{Element, Length};
    use iced_lazy::Component;

    pub struct NumericInput<Message> {
        value: Option<u32>,
        on_change: Box<dyn Fn(Option<u32>) -> Message>,
    }

    pub fn numeric_input<Message>(
        value: Option<u32>,
        on_change: impl Fn(Option<u32>) -> Message + 'static,
    ) -> NumericInput<Message> {
        NumericInput::new(value, on_change)
    }

    #[derive(Debug, Clone)]
    pub enum Event {
        InputChanged(String),
        IncrementPressed,
        DecrementPressed,
    }

    impl<Message> NumericInput<Message> {
        pub fn new(
            value: Option<u32>,
            on_change: impl Fn(Option<u32>) -> Message + 'static,
        ) -> Self {
            Self {
                value,
                on_change: Box::new(on_change),
            }
        }
    }

    impl<Message, Renderer> Component<Message, Renderer> for NumericInput<Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
        Renderer::Theme:
            widget::button::StyleSheet + widget::text_input::StyleSheet + widget::text::StyleSheet,
    {
        type State = ();
        type Event = Event;

        fn update(&mut self, _state: &mut Self::State, event: Event) -> Option<Message> {
            match event {
                Event::IncrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_add(1),
                ))),
                Event::DecrementPressed => Some((self.on_change)(Some(
                    self.value.unwrap_or_default().saturating_sub(1),
                ))),
                Event::InputChanged(value) => {
                    if value.is_empty() {
                        Some((self.on_change)(None))
                    } else {
                        value.parse().ok().map(Some).map(self.on_change.as_ref())
                    }
                }
            }
        }

        fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
            let button = |label, on_press| {
                button(
                    text(label)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .vertical_alignment(alignment::Vertical::Center),
                )
                .width(40)
                .height(40)
                .on_press(on_press)
            };

            row![
                button("-", Event::DecrementPressed),
                text_input(
                    "Type a number",
                    self.value
                        .as_ref()
                        .map(u32::to_string)
                        .as_deref()
                        .unwrap_or(""),
                )
                .on_input(Event::InputChanged)
                .padding(10),
                button("+", Event::IncrementPressed),
            ]
            .align_items(Alignment::Center)
            .spacing(10)
            .into()
        }
    }

    impl<'a, Message, Renderer> From<NumericInput<Message>> for Element<'a, Message, Renderer>
    where
        Message: 'a,
        Renderer: 'static + iced_native::text::Renderer,
        Renderer::Theme:
            widget::button::StyleSheet + widget::text_input::StyleSheet + widget::text::StyleSheet,
    {
        fn from(numeric_input: NumericInput<Message>) -> Self {
            iced_lazy::component(numeric_input)
        }
    }
}
