use nih_plug::log::debug;
use nih_plug::prelude::{AtomicF32, Editor};
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use shaper_view::ShaperView;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};

use crate::MathshaperParams;

use crate::shaper::Shaper as GenericShaper;
const TABLE_SIZE: usize = 512;
type DisplayShaper = GenericShaper<TABLE_SIZE>;
use crate::Shaper as DspShaper;

mod shaper_view;

#[derive(Lens)]
struct Data {
    _params: Arc<MathshaperParams>,
    shaper: Arc<Mutex<DisplayShaper>>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<DspShaper>>>,
}

enum EditorEvent {
    Generate,
    Normalize,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(
            |event: &EditorEvent, _| match event {
                EditorEvent::Generate => {
                    let text_file = File::open(std::env!("TEXT_INPUT_PATH")).unwrap();
                    let mut reader = BufReader::new(text_file);
                    let mut prompt = String::new();
                    reader.read_to_string(&mut prompt).unwrap();

                    let mut lock = self.shaper.lock().unwrap(); // TODO: Error Handling Poison Error
                    lock.prompt(&prompt).unwrap(); // TODO: Error Handling Prompt Error

                    let mut lock = self.shaper_input_data.lock().unwrap();
                    let shaper_input = lock.input_buffer();
                    shaper_input.prompt(&prompt).unwrap(); // TODO: Error Handling Prompt Error
                    lock.publish();
                }
                EditorEvent::Normalize => {
                    let mut lock = self.shaper.lock().unwrap(); // TODO: Error Handling Poison Error
                    lock.normalize();

                    let mut lock = self.shaper_input_data.lock().unwrap(); // TODO: Error Handling Poison Error
                    let shaper_input = lock.input_buffer();
                    shaper_input.normalize();
                    lock.publish();
                }, 
            },
        )
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (900, 450))
}

pub(crate) fn create(
    params: Arc<MathshaperParams>,
    editor_state: Arc<ViziaState>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<DspShaper>>>,
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
            shaper: shaper.clone(),
            peak_max: peak_max.clone(),
            peak_min: peak_min.clone(),
            shaper_input_data: shaper_input_data.clone(),
        }
        .build(cx);

        HStack::new(cx, move |cx| {
            VStack::new(cx, move |cx| {
                Label::new(cx, "PRE");
                Button::new(
                    cx,
                    |cx| {
                        cx.emit(EditorEvent::Generate);
                    },
                    |cx| Label::new(cx, "Reload"),
                );
                Button::new(
                    cx,
                    |cx| cx.emit(EditorEvent::Normalize),
                    |cx| Label::new(cx, "Normalize"),
                );
            })
            .class("side-container");

            VStack::new(cx, move |cx| {
                ShaperView::new(cx, Data::shaper, Data::peak_max, Data::peak_min);
                // TODO: Resizing layout, keep at square
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
