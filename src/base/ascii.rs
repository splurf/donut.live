use artem::ConfigBuilder;
use bincode::{deserialize, serialize};
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, AnimationDecoder, DynamicImage, Frame,
    ImageDecoder,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    fs::{read, File},
    io::{BufReader, Write},
    path::Path,
    time::Duration,
};

use super::{donut, Config, GifError, Result};

#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
    width: u32,
    height: u32,
}

impl Dimensions {
    const fn w(&self) -> u32 {
        self.width
    }

    const fn h(&self) -> u32 {
        self.height
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        self.buffer.as_slice()
    }
}

pub fn frame_to_ascii(
    f: Frame,
    delay: Option<Duration>,
    dims: Option<Dimensions>,
    config: &artem::config::Config,
) -> AsciiFrame {
    // use specified delay or delay of the current frame
    let delay = delay.unwrap_or(f.delay().into());

    // represent image buffer as dynamic image
    let mut image = DynamicImage::from(f.into_buffer());

    if let Some(dims) = dims {
        // resize image dimensions based on provided dimensions
        image = image.resize_exact(dims.w(), dims.h(), FilterType::Nearest);
    }

    // convert image into ASCII art
    let s = artem::convert(image, config);

    // buffer and delay data only
    AsciiFrame {
        buffer: s.into_bytes(),
        delay,
    }
}

fn get_frames_from_path(
    path: &Path,
    fps: Option<f32>,
    is_colored: bool,
) -> Result<Vec<AsciiFrame>> {
    // file reader
    let input = BufReader::new(File::open(path)?);

    // configure the decoder such that it will expand the image to RGBA
    let decoder = GifDecoder::new(input)?;

    // clamp dimensions
    let (width, height) = decoder.dimensions();
    let dims = (height > 56).then_some(Dimensions {
        width: (width as f32 * (56.0 / height as f32)) as u32,
        height: 56,
    });

    // pass through color determinant
    let config = ConfigBuilder::new().color(is_colored).build();

    // Read the file header
    let frames = decoder.into_frames().collect_frames()?;

    // determine delay between each frame
    let delay = fps.map(|value| Duration::from_secs_f32(1.0 / value));

    // convert frame buffer to ASCII, extract buffer and delay only
    let ascii = frames
        .into_par_iter()
        .map(|f| frame_to_ascii(f, delay, dims, &config))
        .collect::<Vec<AsciiFrame>>();

    if ascii.iter().all(|f| f.delay().is_zero()) {
        return Err(GifError::Delay.into());
    }
    Ok(ascii)
}

pub fn read_file(file_name: &str) -> Result<Vec<AsciiFrame>> {
    // read contents of file
    let bytes = read(file_name)?;

    // deserialize each frame
    let mut frames = deserialize::<Vec<AsciiFrame>>(&bytes)?;
    for frame in frames.iter_mut() {
        // preprend home ascii escape sequence to each frame buffer
        frame.buffer.splice(0..0, "\x1b[H".bytes());
    }
    Ok(frames)
}

pub fn write_file(
    gif: Option<&Path>,
    fps: Option<f32>,
    is_colored: bool,
    file_name: &str,
) -> Result<()> {
    // generate frames
    let frames = if let Some(path) = gif {
        get_frames_from_path(path, fps, is_colored)?
    } else {
        donut::get_frames() // default
    };

    let bytes = serialize(&frames)?;

    // write to file
    let mut file = File::create(file_name)?;
    file.write_all(&bytes).map_err(Into::into)
}

pub fn get_frames(cfg: &Config) -> Result<Vec<AsciiFrame>> {
    // construct new file name
    let file_name = cfg.file_name();

    // generate and write frames to file if they don't already exist
    read_file(&file_name)
        .or_else(|_| {
            // save to file
            write_file(cfg.gif(), cfg.fps(), cfg.is_colored(), &file_name)?;

            // reread from file
            read_file(&file_name)
        })
        .map_err(Into::into)
}
