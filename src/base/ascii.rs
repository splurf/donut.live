use artem::ConfigBuilder;
use image::{codecs::gif::GifDecoder, AnimationDecoder, DynamicImage, Frame};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs::File, io::BufReader, path::Path, time::Duration};

use super::{donut, Config, GifError, Result};

#[derive(Clone, Debug)]
pub struct AsciiFrame {
    buffer: Vec<u8>,
    delay: Duration,
}

impl AsciiFrame {
    pub const fn new(buffer: Vec<u8>, delay: Duration) -> Self {
        Self { buffer, delay }
    }

    pub const fn delay(&self) -> Duration {
        self.delay
    }
}

impl AsRef<[u8]> for AsciiFrame {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

pub fn frame_to_ascii(
    f: Frame,
    delay: Option<Duration>,
    config: &artem::config::Config,
) -> AsciiFrame {
    // use specified delay or delay of the current frame
    let delay = delay.unwrap_or(f.delay().into());

    // represent image buffer as dynamic image
    let image = DynamicImage::from(f.into_buffer());

    // convert image into ASCII art
    let s = artem::convert(image, config);

    // prepend HOME ASCII escape sequence
    let mut buffer = b"\x1b[H".to_vec();
    buffer.extend(s.as_bytes());

    // buffer and delay data only
    AsciiFrame { buffer, delay }
}

fn get_frames_from_path(
    path: &Path,
    fps: Option<f32>,
    is_colored: bool,
) -> Result<Vec<AsciiFrame>> {
    // file reader
    let input = BufReader::new(File::open(path)?);

    // Configure the decoder such that it will expand the image to RGBA.
    let decoder = GifDecoder::new(input).unwrap();

    // Read the file header
    let frames = decoder.into_frames().collect_frames().unwrap();

    let delay = fps.map(|value| Duration::from_secs_f32(1.0 / value));
    let config = ConfigBuilder::new().color(is_colored).build();

    // convert frame buffer to ASCII, extract buffer and delay only
    let ascii = frames
        .into_par_iter()
        .map(|f| frame_to_ascii(f, delay, &config))
        .collect::<Vec<AsciiFrame>>();

    if ascii.iter().all(|f| f.delay().is_zero()) {
        return Err(GifError::Delay.into());
    }
    Ok(ascii)
}

pub fn get_frames(cfg: &Config) -> Result<Vec<AsciiFrame>> {
    if let Some(path) = cfg.gif() {
        get_frames_from_path(path, cfg.fps(), cfg.colored())
    } else {
        Ok(donut::get_frames())
    }
}
