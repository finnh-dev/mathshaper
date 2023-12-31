use nih_plug::prelude::{Editor, GuiContext};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;



use crate::MathshaperParams;
use crate::shaper::Shaper;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(300, 300)
}

pub(crate) fn create(
    params: Arc<MathshaperParams>,
    shaper: Arc<Shaper>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<MathshaperEditor>(editor_state, (params, shaper))
}

struct MathshaperEditor {
    params: Arc<MathshaperParams>,
    context: Arc<dyn GuiContext>,

    shaper: Arc<Shaper>,

    prompt: String,

    prompt_input_state: nih_plug_iced::text_input::State,
    dry_slider_state: nih_widgets::param_slider::State,
    wet_slider_state: nih_widgets::param_slider::State,
}

#[derive(Debug, Clone)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),
    InputChanged(String),
}

impl MathshaperEditor {
    fn handle_prompt_input(&mut self, input: String) {
        self.prompt = input;
    }
}

impl IcedEditor for MathshaperEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<MathshaperParams>, Arc<Shaper>);

    fn new(
        (params, shaper): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = MathshaperEditor {
            params,
            shaper,
            context,

            prompt: String::new(),

            prompt_input_state: Default::default(),
            dry_slider_state: Default::default(),
            wet_slider_state: Default::default(),
        };

        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::ParamUpdate(message) => self.handle_param_message(message),
            Message::InputChanged(input) => self.handle_prompt_input(input),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .align_items(Alignment::Center)
            .push(
                Text::new("Mathshaper GUI")
                    .font(assets::NOTO_SANS_LIGHT)
                    .size(40)
                    .height(50.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Bottom),
            )
            .push(
                Text::new("Dry")
                    .height(20.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.dry_slider_state, &self.params.dry)
                    .map(Message::ParamUpdate),
            )
            .push(
                Text::new("Wet")
                    .height(20.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Center),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.wet_slider_state, &self.params.wet)
                    .map(Message::ParamUpdate),
            )
            .push(Space::with_height(10.into()))
            .push(
                TextInput::new(&mut self.prompt_input_state, "Prompt...", &self.prompt, Message::InputChanged)
            )
            .into()
    }

    fn background_color(&self) -> nih_plug_iced::Color {
        nih_plug_iced::Color {
            r: 0.98,
            g: 0.98,
            b: 0.98,
            a: 1.0,
        }
    }
}