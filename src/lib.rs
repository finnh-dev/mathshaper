mod editor;
mod math;
mod shaper;

use core::f32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use shaper::Shaper as GenericShaper;
use std::sync::{Arc, Mutex};
use triple_buffer::TripleBuffer;
use valib::oversample::Oversample;
// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

type Shaper = GenericShaper<512>; // TODO: Figure out size

const MAX_BLOCK_SIZE: usize = 512;
const OVERSAMPLE_MAX: usize = 16;

pub struct Mathshaper {
    params: Arc<MathshaperParams>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
    shaper_input_data: Arc<Mutex<triple_buffer::Input<Shaper>>>,
    shaper_output_data: triple_buffer::Output<Shaper>,
    resamplers: Box<[Oversample<f32>]>,
}

#[derive(Params)]
struct MathshaperParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[id = "pre_gain"]
    pub pre_gain: FloatParam,
    #[id = "post_gain"]
    pub post_gain: FloatParam,
    #[id = "decay"]
    pub decay: FloatParam,
}

impl Default for Mathshaper {
    fn default() -> Self {
        let (shaper_in, shaper_out) = TripleBuffer::default().split();
        Self {
            params: Arc::new(MathshaperParams::default()),
            peak_max: Arc::default(),
            peak_min: Arc::default(),
            shaper_input_data: Arc::new(Mutex::new(shaper_in)),
            shaper_output_data: shaper_out,
            resamplers: vec![].into_boxed_slice(),
        }
    }
}

impl Default for MathshaperParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
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
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            post_gain: FloatParam::new(
                "Post Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(10.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 10.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            decay: FloatParam::new("decay", 0.4, FloatRange::Linear { min: 0.0, max: 3.0 }),
        }
    }
}

impl Plugin for Mathshaper {
    const NAME: &'static str = "Mathshaper";
    const VENDOR: &'static str = "Finn Heintzmann";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "f.heintzmann@ostfalia.de";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
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
        let resamplers = vec![Oversample::<f32>::new(OVERSAMPLE_MAX, MAX_BLOCK_SIZE); input_channels];
        self.resamplers = resamplers.into_boxed_slice();
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let pre_gain = self.params.pre_gain.smoothed.next();
        let post_gain = self.params.post_gain.smoothed.next();

        let mut new_peak_max = f32::MIN;
        let mut new_peak_min = f32::MAX;

        let shaper_data = self.shaper_output_data.read();

        // for (_, mut block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
        //     for channel in 0..block.channels() {
        //         let io_buffer = block.get_mut(channel).expect("Channel index out of bounds");
        //         let mut oversampled_block = self.resamplers[channel].oversample(io_buffer);

        //         for sample in oversampled_block.iter_mut() {
        //             *sample = *sample * pre_gain;
        //             new_peak_max = new_peak_max.max(*sample);
        //             new_peak_min = new_peak_min.min(*sample);
        //             *sample = shaper_data.process(*sample) * post_gain;
        //         }

        //         oversampled_block.finish(io_buffer);
        //     }
        // }

        for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            for (channel, io_buffer) in block.into_iter().enumerate() {

                if channel >= self.resamplers.len() {
                    nih_log!("Channel index out of bounds");
                    break;
                }
            
                let mut oversampled_block = self.resamplers[channel].oversample(io_buffer);

                for sample in oversampled_block.iter_mut() {
                    *sample = *sample * pre_gain;
                    new_peak_max = new_peak_max.max(*sample);
                    new_peak_min = new_peak_min.min(*sample);
                    *sample = shaper_data.process(*sample) * post_gain;
                }

                oversampled_block.finish(io_buffer);
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

        // for channel_samples in buffer.iter_samples() {
        //     // Smoothing is optionally built into the parameters themselves

        //     // let oversampled = self.resampler.oversample(channel_samples.)

        //     for sample in channel_samples {
        //         *sample = *sample * pre_gain;
        //         new_peak_max = new_peak_max.max(*sample);
        //         new_peak_min = new_peak_min.min(*sample);
        //         *sample = shaper_data.process(*sample) * post_gain;
        //     }

        // }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Mathshaper {
    const CLAP_ID: &'static str = "com.finnh.mathshaper";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Mathshaper {
    const VST3_CLASS_ID: [u8; 16] = *b"mathshaperfinnhe";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Mathshaper);
nih_export_vst3!(Mathshaper);
