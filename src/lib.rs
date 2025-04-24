mod editor;
mod math;
mod shaper;

use core::f32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use shaper::{compile_shaper, Shaper};
use std::sync::{Arc, Mutex, RwLock};
use triple_buffer::TripleBuffer;
use valib::oversample::Oversample;

const MAX_BLOCK_SIZE: usize = 512;
const OVERSAMPLE_MAX: usize = 16;

pub struct Mathshaper {
    params: Arc<MathshaperParams>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<Arc<Shaper>>>>,
    shaper_output_data: triple_buffer::Output<Arc<Shaper>>,
    resamplers: Box<[Oversample<f32>]>,
    expression: Arc<RwLock<String>>,
}

#[derive(Params)]
struct MathshaperParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[id = "pre_gain"]
    pub pre_gain: FloatParam,
    #[id = "post_gain"]
    pub post_gain: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
    #[id = "a"]
    pub a: FloatParam,
    #[id = "b"]
    pub b: FloatParam,
    #[id = "c"]
    pub c: FloatParam,
    #[id = "d"]
    pub d: FloatParam,
}

impl Default for Mathshaper {
    fn default() -> Self {
        let function = compile_shaper("x").expect("Default function should compile");
        let (shaper_in, shaper_out) = TripleBuffer::new(&Arc::new(function)).split();
        Self {
            params: Arc::new(MathshaperParams::default()),
            peak_max: Arc::default(),
            peak_min: Arc::default(),
            shaper_input_data: Arc::new(Mutex::new(shaper_in)),
            shaper_output_data: shaper_out,
            resamplers: vec![].into_boxed_slice(),
            expression: Arc::new(RwLock::new("x".to_owned())),
        }
    }
}

impl Default for MathshaperParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            pre_gain: FloatParam::new(
                "Pre Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(10.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 10.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            post_gain: FloatParam::new(
                "Post Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(10.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 10.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            decay: FloatParam::new("decay", 0.4, FloatRange::Linear { min: 0.0, max: 3.0 }),
            a: FloatParam::new("a", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            b: FloatParam::new("b", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            c: FloatParam::new("c", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
            d: FloatParam::new("d", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 }),
        }
    }
}

impl Plugin for Mathshaper {
    const NAME: &'static str = "Mathshaper";
    const VENDOR: &'static str = "Finn Heintzmann";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "f.heintzmann@ostfalia.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
            self.peak_max.clone(),
            self.peak_min.clone(),
            self.shaper_input_data.clone(),
            self.expression.clone(),
        )
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        println!("input channels: {:?}", audio_io_layout.main_output_channels);
        let input_channels = audio_io_layout
            .main_input_channels
            .unwrap_or(unsafe { NonZeroU32::new_unchecked(1) })
            .get() as usize;
        let resamplers =
            vec![Oversample::<f32>::new(OVERSAMPLE_MAX, MAX_BLOCK_SIZE); input_channels];
        self.resamplers = resamplers.into_boxed_slice();
        true
    }

    fn reset(&mut self) {

    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut new_peak_max = f32::MIN;
        let mut new_peak_min = f32::MAX;

        let shaper_data = self.shaper_output_data.read();

        for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            for (channel, io_buffer) in block.into_iter().enumerate() {
                if channel >= self.resamplers.len() {
                    nih_log!("Channel index out of bounds");
                    break;
                }

                // let mut oversampled_block = self.resamplers[channel].oversample(io_buffer);

                let pre_gain = self.params.pre_gain.smoothed.next();
                let post_gain = self.params.post_gain.smoothed.next();

                let a = self.params.a.value();
                let b = self.params.b.value();
                let c = self.params.c.value();
                let d = self.params.d.value();

                for sample in io_buffer.iter_mut() {
                    *sample *= pre_gain;
                    new_peak_max = new_peak_max.max(*sample);
                    new_peak_min = new_peak_min.min(*sample);
                    *sample = shaper_data(*sample, a, b, c, d) * post_gain; // TODO: add params
                }

                // oversampled_block.finish(io_buffer);
            }
        }

        let old_peak_max = self.peak_max.load(std::sync::atomic::Ordering::Relaxed);
        let old_peak_min = self.peak_min.load(std::sync::atomic::Ordering::Relaxed);
        let decay = self.params.decay.value() / 100.0; // TODO: Improve decay

        let peak_max = if new_peak_max > old_peak_max {
            new_peak_max
        } else {
            old_peak_max * (1.0 - decay)
        };
        let peak_min = if new_peak_min < old_peak_min {
            new_peak_min
        } else {
            old_peak_min * (1.0 - decay)
        };

        self.peak_max
            .store(peak_max, std::sync::atomic::Ordering::Relaxed);
        self.peak_min
            .store(peak_min, std::sync::atomic::Ordering::Relaxed);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Mathshaper {
    const CLAP_ID: &'static str = "com.finnh.mathshaper";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Mathshaper {
    const VST3_CLASS_ID: [u8; 16] = *b"mathshaperfinnhe";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Mathshaper);
nih_export_vst3!(Mathshaper);
