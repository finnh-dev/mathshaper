use nih_plug::log::debug;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use shaper_view::ShaperView;
use std::sync::{Arc, Mutex};

use crate::MathshaperParams;

use crate::shaper::Shaper as GenericShaper;

const TABLE_SIZE: usize = 512;

type Shaper = GenericShaper<TABLE_SIZE>;

mod shaper_view;

#[derive(Lens)]
struct Data {
    _params: Arc<MathshaperParams>,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, _event: &mut Event) {
        
    }
}

#[derive(Lens)]
struct TextInputData {
    input: String,
    shaper: Arc<Mutex<Shaper>>
}

impl Model for TextInputData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(
            |text_input_event: &PromptInputEvent, _| match text_input_event {
                PromptInputEvent::PromptChanged(input) => {
                    self.input = input.to_owned();
                },
                PromptInputEvent::Generate => {
                    let mut lock = self.shaper.lock().unwrap(); // TODO: Remove unwrap
                    lock.prompt(&self.input).unwrap(); // TODO: Remove unwrap
                }
            },
        )
    }
}
enum PromptInputEvent {
    PromptChanged(String),
    Generate
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (900, 450))
}

pub(crate) fn create(
    params: Arc<MathshaperParams>,
    editor_state: Arc<ViziaState>, 
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        debug!("Creating view...");
        // assets::register_noto_sans_light(cx);
        // assets::register_noto_sans_thin(cx);
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("Failed to load stylesheet");

        let shaper = Arc::new(Mutex::default());
        Data {
            _params: params.clone(),
        }
        .build(cx);

        HStack::new(cx, move |cx| {
            let shaper_copy = shaper.clone();
            VStack::new(cx, move|cx| {
                Label::new(cx, "PRE");
                Button::new(cx, move |_cx| {
                    prompt_input_modal(shaper_copy.clone());
                }, |cx| Label::new(cx, "Edit"));
            })
            .class("side-container");

            VStack::new(cx, move |cx| {
                ShaperView::new(cx, shaper.clone()); // TODO: Resizing layout, keep at square
            })
            .class("main-container");

            VStack::new(cx, |cx| {
                Label::new(cx, "POST");
            })
            .class("side-container");
        })
        .class("main-row");
    })
}

fn prompt_input_modal(shaper: Arc<Mutex<Shaper>>) {
    std::thread::spawn(move || {
        Application::new(move |cx| {
            TextInputData {
                input: String::new(),
                shaper: shaper.clone()
            }
            .build(cx);
            Textbox::new(cx, TextInputData::input)
                .on_edit(move |cx, text| cx.emit(PromptInputEvent::PromptChanged(text)))
                .width(Pixels(200.0))
                .height(Pixels(30.0));
            Button::new(cx, |cx| {
                cx.emit(PromptInputEvent::Generate);
            },
            |cx| Label::new(cx, "Generate"));
        })
        .title("Text Input")
        .inner_size(WindowSize {
            width: 200,
            height: 60,
        })
        .run();
    });
}