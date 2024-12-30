use nih_plug::log::debug;
use nih_plug::prelude::{AtomicF32, Editor};
use nih_plug_vizia::vizia::prelude::*;

use nih_plug_vizia::widgets::ParamSlider;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use shaper_view::ShaperView;
use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex, RwLock};

use crate::shaper::{compile_shaper, Shaper};
use crate::MathshaperParams;

mod shaper_view;

#[derive(Lens)]
struct Data {
    params: Arc<MathshaperParams>,
    shaper: Arc<Mutex<Arc<Shaper>>>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<Arc<Shaper>>>>,
    last_error: String,
    expression: Arc<RwLock<String>>,
}

enum EditorEvent {
    Generate,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|event: &EditorEvent, _| match event {
            EditorEvent::Generate => {
                let text_file = File::open(std::env!("TEXT_INPUT_PATH")).unwrap();
                let mut reader = BufReader::new(text_file);
                let mut prompt = String::new();
                reader.read_to_string(&mut prompt).unwrap();

                println!("Prompt: {prompt}");

                let func = match compile_shaper(prompt.clone()) {
                    Ok(f) => {
                        let mut lock = self.expression.write().expect("Poisond lock");
                        *lock = prompt;
                        f
                    }
                    Err(e) => {
                        self.last_error = format!("Error:\n{:#?}", e);
                        return;
                    }
                };

                let ts_func = Arc::new(func);

                let mut lock = self.shaper.lock().expect("Poisoned Mutex");
                *lock = ts_func.clone();

                let mut lock = self.shaper_input_data.lock().unwrap(); // TODO: Error Handling Poison Error
                let shaper_input = lock.input_buffer();
                *shaper_input = ts_func.clone();
                lock.publish();

                self.last_error.clear();
            }
        })
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (900, 540))
}

pub(crate) fn create(
    params: Arc<MathshaperParams>,
    editor_state: Arc<ViziaState>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<Arc<Shaper>>>>,
    expression: Arc<RwLock<String>>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        debug!("Creating view...");
        // assets::register_noto_sans_light(cx);
        // assets::register_noto_sans_thin(cx);
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("Failed to load stylesheet");

        let expr = expression.read().expect("Poisoned Lock").clone();
        let func = compile_shaper(expr).expect("Failed to compile stored expression");
        let shaper = Arc::new(Mutex::new(Arc::new(func)));
        Data {
            params: params.clone(),
            shaper: shaper.clone(),
            peak_max: peak_max.clone(),
            peak_min: peak_min.clone(),
            shaper_input_data: shaper_input_data.clone(),
            last_error: String::new(),
            expression: expression.clone(),
        }
        .build(cx);

        VStack::new(cx, move |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "a:");
                    ParamSlider::new(cx, Data::params, |params| &params.a);
                })
                .width(Stretch(1.0));
                VStack::new(cx, |cx| {
                    Label::new(cx, "b:");
                    ParamSlider::new(cx, Data::params, |params| &params.b);
                })
                .width(Stretch(1.0));
                VStack::new(cx, |cx| {
                    Label::new(cx, "c:");
                    ParamSlider::new(cx, Data::params, |params| &params.c);
                })
                .width(Stretch(1.0));
                VStack::new(cx, |cx| {
                    Label::new(cx, "d:");
                    ParamSlider::new(cx, Data::params, |params| &params.d);
                })
                .width(Stretch(1.0));
            })
            .height(Stretch(1.0));
            HStack::new(cx, move |cx| {
                VStack::new(cx, move |cx| {
                    Label::new(cx, "PRE").width(Stretch(1.0));
                    Button::new(
                        cx,
                        |cx| {
                            cx.emit(EditorEvent::Generate);
                        },
                        |cx| Label::new(cx, "Reload"),
                    )
                    .width(Stretch(1.0));
                    Label::new(cx, Data::last_error)
                        .width(Stretch(1.0))
                        .height(Stretch(4.0))
                        .text_wrap(true);
                })
                .class("side-container");

                VStack::new(cx, move |cx| {
                    ShaperView::new(
                        cx,
                        Data::shaper,
                        Data::peak_max,
                        Data::peak_min,
                        Data::params,
                    );
                    // TODO: Resizing layout, keep at square
                })
                .class("main-container");

                VStack::new(cx, |cx| {
                    Label::new(cx, "POST");
                })
                .class("side-container");
            })
            .class("main-row")
            .height(Stretch(5.0));
        });
    })
}
