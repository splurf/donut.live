use artem::ConfigBuilder;
use bincode::{deserialize, serialize};
use gif::DecodeOptions;
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, AnimationDecoder, DynamicImage, ImageDecoder,
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use log::info;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    fs::{read, File},
    io::{BufReader, BufWriter, Read, Seek, SeekFrom},
    path::Path,
    time::Duration,
};
use zstd::{decode_all, stream::copy_encode, zstd_safe::max_c_level};

use super::{donut, Config, GifError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsciiFrame {
    #[serde(with = "serde_bytes")]
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

fn frame_to_ascii(
    f: image::Frame,
    delay: Option<Duration>,
    dims: Option<(u32, u32)>,
    config: &artem::config::Config,
) -> AsciiFrame {
    // use specified delay or delay of the current frame
    // let delay = delay.unwrap_or(Duration::from_millis(f.delay.into()));
    let delay = delay.unwrap_or(f.delay().into());

    // represent image buffer as dynamic image
    let mut image = DynamicImage::from(f.into_buffer());

    // resize if needed
    if let Some((w, h)) = dims {
        // resize image dimensions based on provided dimensions
        image = image.resize_exact(w, h, FilterType::Nearest);
    }

    // convert image into ASCII art
    let s = artem::convert(image, config);

    // buffer and delay data only
    AsciiFrame {
        buffer: s.into_bytes(),
        delay,
    }
}

fn get_frames_count(input: &mut BufReader<File>) -> Result<u64> {
    // decoder configuration
    let mut options = DecodeOptions::new();

    info!("Counting frames");
    // determine number of frames
    options.skip_frame_decoding(true);
    let mut count = 0;

    // iterate through frames without decoding
    let mut decoder = options.read_info(input.by_ref())?;

    // count each image
    (0..)
        .try_for_each(|_| decoder.next_frame_info().ok()?.map(|_| count += 1))
        .into_par_iter();
    info!("Number of frames: {}", count);

    Ok(count)
}

fn get_ascii_frames(
    input: BufReader<File>,
    fps: Option<f32>,
    is_colored: bool,
    count: u64,
) -> Result<Vec<AsciiFrame>> {
    // init 'image' decoder
    let decoder = GifDecoder::new(input)?;

    // determine delay between each frame
    let delay = fps.map(|value| Duration::from_secs_f32(1.0 / value));

    // regulate then clamp dimensions
    let dims = {
        let (w, h) = decoder.dimensions();
        (h > 56).then_some(((w as f32 * (56.0 / h as f32)) as u32, 56))
    };

    // provide the color determinant
    let config = ConfigBuilder::new().color(is_colored).build();

    info!("Decoding frames");
    // decode frames
    let frames = decoder
        .into_frames()
        .progress_count(count)
        .collect::<Result<Vec<image::Frame>, _>>()?;

    info!("Converting frames info ASCII");
    // convert frame buffer to ASCII, extract buffer and delay only
    Ok(frames
        .into_par_iter()
        .progress_count(count)
        .map(|f| frame_to_ascii(f, delay, dims, &config))
        .collect())
}

fn get_frames_from_path(
    path: &Path,
    fps: Option<f32>,
    is_colored: bool,
) -> Result<Vec<AsciiFrame>> {
    // file reader
    let mut input = BufReader::new(File::open(path)?);

    // determine number of frames
    let count = get_frames_count(input.by_ref())?;

    // seek back to beginning of file
    input.seek(SeekFrom::Start(0))?;

    // convert frames into ASCII
    let ascii = get_ascii_frames(input, fps, is_colored, count)?;

    info!("Validating frames");
    // ensure frame delays are consistent
    if ascii
        .par_iter()
        .progress_count(count)
        .all(|f| f.delay().is_zero())
    {
        return Err(GifError::Delay.into());
    }
    Ok(ascii)
}

fn read_file(file_name: &str) -> Result<Vec<AsciiFrame>> {
    // read contents of file
    info!("Looking for {:?} file", file_name);
    let src = read(file_name)?;

    // decompress contents
    info!("Decompressing data");
    let decompressed = decode_all(src.as_slice())?;

    // deserialize each frame
    info!("Deserializing data");
    deserialize(&decompressed).map_err(Into::into)
}

pub fn write_file(
    gif: Option<&Path>,
    fps: Option<f32>,
    is_colored: bool,
    file_name: &str,
) -> Result<Vec<AsciiFrame>> {
    // generate frames
    let frames = if let Some(path) = gif {
        info!("Converting frames from {:?}", path);
        get_frames_from_path(path, fps, is_colored)?
    } else {
        info!("Generating donut frames");
        donut::get_frames() // default
    };
    // serialize to bytes
    info!("Serializing data");
    let src = serialize(&frames.clone())?;

    // write to file while compressing serialization
    info!("Writing to file");
    let dst = BufWriter::new(File::create(file_name)?);
    copy_encode(src.as_slice(), dst, max_c_level())?;

    // return the generated frames
    Ok(frames)
}

pub fn get_frames(cfg: &Config) -> Result<Vec<AsciiFrame>> {
    info!("Retrieving ASCII frames");

    // generate and write frames to file if they don't already exist
    let mut frames = read_file(cfg.file_name()).or_else(|_| {
        // save to file, while returning original result
        write_file(cfg.gif(), cfg.fps(), cfg.is_colored(), cfg.file_name())
    })?;

    // preprend home ascii escape sequence to each frame buffer
    info!("Finishing frames");
    for frame in frames.iter_mut() {
        frame.buffer.splice(0..0, "\x1b[H".bytes());
    }
    Ok(frames)
}
