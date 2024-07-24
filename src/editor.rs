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
    params: Arc<MathshaperParams>,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, _event: &mut Event) {
        // debug!("Editor event called: {:?}", event);
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
            params: params.clone(),
        }
        .build(cx);

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(cx, "PRE");
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
