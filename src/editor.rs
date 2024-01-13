use nih_plug::log::debug;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

use crate::MathshaperParams;

#[derive(Lens)]
struct Data {
    params: Arc<MathshaperParams>,
    prompt_input: String,
    show_modal: bool,
}

// enum EditorEvent {

// }

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        debug!("Editor event called: {:?}", event);
        // event.map(|editor_event, _|
        //     match editor_event {
        //         // EditorEvent::PromptChanged(input) => {
        //         //     self.prompt_input = input.clone();
        //         // }
        //     }
        // )
    }
}

#[derive(Lens)]
struct TextInputData {
    prompt_input: String,
}

impl Model for TextInputData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        debug!("Editor event called: {:?}", event);
        event.map(
            |text_input_event: &PromptInputEvent, _| match text_input_event {
                PromptInputEvent::PromptChanged(input) => self.prompt_input = input.to_owned(),
            },
        )
    }
}
enum PromptInputEvent {
    PromptChanged(String),
}
// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (300, 300))
}

pub(crate) fn create(
    params: Arc<MathshaperParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    std::thread::spawn(|| {
        Application::new(|cx| {
            TextInputData {
                prompt_input: String::new(),
            }
            .build(cx);
            Textbox::new(cx, TextInputData::prompt_input)
                .on_edit(move |cx, text| cx.emit(PromptInputEvent::PromptChanged(text)))
                .width(Pixels(200.0))
                .height(Pixels(30.0));
        })
        .title("Text Input")
        .inner_size(WindowSize {
            width: 200,
            height: 30,
        })
        .run();
    });

    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        debug!("Creating view...");
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            prompt_input: String::new(),
            show_modal: false,
        }
        .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Mathshaper GUI")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            Label::new(cx, "Wet");
            ParamSlider::new(cx, Data::params, |params| &params.wet);
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));

        ResizeHandle::new(cx);
    })
}
