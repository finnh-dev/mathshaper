#[cfg(feature = "standalone")]
use mathshaper::Mathshaper;
#[cfg(feature = "standalone")]
use nih_plug::nih_export_standalone;

fn main() {
    #[cfg(feature = "standalone")]
    nih_export_standalone::<Mathshaper>();
}
