use nih_plug::log::debug;
use nih_plug::prelude::{AtomicF32, Editor};
use vizia_plug::vizia::prelude::*;

use shaper_view::ShaperView;
// use std::fs::File;
// use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex, RwLock};
use vizia_plug::widgets::ParamSlider;
use vizia_plug::{create_vizia_editor, ViziaState, ViziaTheming};

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
    input_string: String,
}

enum EditorEvent {
    Generate(String),
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|event: &EditorEvent, _| match event {
            EditorEvent::Generate(input) => {
                // let text_file = File::open(std::env!("TEXT_INPUT_PATH")).unwrap();
                // let mut reader = BufReader::new(text_file);
                // let mut prompt = String::new();
                // reader.read_to_string(&mut prompt).unwrap();
                let prompt = input;
                println!("Prompt: {prompt}");

                let func = match compile_shaper(prompt.clone()) {
                    Ok(f) => {
                        let mut lock = self.expression.write().expect("Poisond lock");
                        *lock = prompt.to_string();
                        self.last_error.clear();
                        f
                    }
                    Err(e) => {
                        self.last_error = format!("Error:\n{:#?}", e);
                        return;
                    }
                };
                self.input_string = prompt.to_string();

                let ts_func = Arc::new(func);

                let mut lock = self.shaper.lock().expect("Poisoned Mutex");
                *lock = ts_func.clone();

                let mut lock = self.shaper_input_data.lock().unwrap(); // TODO: Error Handling Poison Error
                let shaper_input = lock.input_buffer_mut();
                *shaper_input = ts_func.clone();
                lock.publish();
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
            input_string: "x".to_string(),
        }
        .build(cx);
        VStack::new(cx, move |cx| {
            Textbox::new(cx, Data::input_string)
                .on_edit(|cx, input| cx.emit(EditorEvent::Generate(input)))
                .width(Stretch(1.0));
            VStack::new(cx, move |cx| {
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "a:");
                        ParamSlider::new(cx, Data::params, |params| &params.a).width(Stretch(1.0));
                    })
                    .width(Stretch(1.0));
                    VStack::new(cx, |cx| {
                        Label::new(cx, "b:");
                        ParamSlider::new(cx, Data::params, |params| &params.b).width(Stretch(1.0));
                    })
                    .width(Stretch(1.0));
                    VStack::new(cx, |cx| {
                        Label::new(cx, "c:");
                        ParamSlider::new(cx, Data::params, |params| &params.c).width(Stretch(1.0));
                    })
                    .width(Stretch(1.0));
                    VStack::new(cx, |cx| {
                        Label::new(cx, "d:");
                        ParamSlider::new(cx, Data::params, |params| &params.d).width(Stretch(1.0));
                    })
                    .width(Stretch(1.0));
                })
                .class("param-container")
                .height(Stretch(1.0));
                HStack::new(cx, move |cx| {
                    VStack::new(cx, move |cx| {
                        Label::new(cx, "PRE").width(Stretch(1.0));
                        ParamSlider::new(cx, Data::params, |params| &params.pre_gain)
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
                            Data::input_string,
                        )
                        .bind(Data::params, |mut handle, _| handle.needs_redraw());
                        // TODO: Resizing layout, keep at square
                    })
                    .class("main-container");

                    VStack::new(cx, |cx| {
                        Label::new(cx, "POST");
                        ParamSlider::new(cx, Data::params, |params| &params.post_gain)
                            .width(Stretch(1.0));
                    })
                    .class("side-container");
                })
                .class("main-row")
                .height(Stretch(5.0));
            });
        })
        .class(".main-row");
    })
}
