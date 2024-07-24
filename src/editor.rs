use nih_plug::log::debug;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use shaper_view::ShaperView;
use std::sync::Arc;

use crate::MathshaperParams;

mod shaper_view;

#[derive(Lens)]
struct Data {
    _params: Arc<MathshaperParams>,
    prompt_input: String,
}

enum EditorEvent {
    PromptChanged(String),
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        debug!("Editor event called: {:?}", event);
        event.map(|editor_event, _| match editor_event {
            EditorEvent::PromptChanged(input) => {
                self.prompt_input = input.clone();
                // TODO: Change ShaperView
            }
        })
    }
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

        Data {
            _params: params.clone(),
            prompt_input: String::new(),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "PRE");
                Textbox::new(cx, Data::prompt_input)
                    .on_edit(|cx, input| cx.emit(EditorEvent::PromptChanged(input)));
            })
            .class("side-container");

            VStack::new(cx, |cx| {
                ShaperView::new(cx); // TODO: Resizing layout, keep at square
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
