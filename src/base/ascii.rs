use artem::ConfigBuilder;
use image::{
    codecs::gif::GifDecoder, imageops::FilterType, AnimationDecoder, DynamicImage, Frame,
    ImageDecoder,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    fs::{read, File},
    io::{BufReader, Write},
    path::Path,
    time::Duration,
    vec::IntoIter,
};

use super::{donut, Config, Error, GifError, Result};

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

fn read_buf<const N: usize>(iter: &mut IntoIter<u8>) -> Option<[u8; N]> {
    let mut buf = [0; N];
    for byte in buf.iter_mut() {
        *byte = iter.next()?
    }
    Some(buf)
}

#[repr(C)]
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

    pub fn into_bytes(self) -> Vec<u8> {
        // buffer length as a 'usize'
        let buffer_len = self.buffer.len();

        // duration as milliseconds as a 'u64'
        let millis = self.delay.as_millis() as u64;

        // frame buffer => [<LENGTH [usize]>, <DELAY [u64]>, <RAW_BUFFER [u8; LENGTH]>]
        let mut data = Vec::with_capacity(size_of::<usize>() + size_of::<u64>() + buffer_len);

        // put it all together
        data.extend(buffer_len.to_ne_bytes());
        data.extend(millis.to_ne_bytes());
        data.extend(self.buffer);

        data
    }

    pub fn from_bytes(iter: &mut IntoIter<u8>) -> Option<Self> {
        // deserialize buffer length as 'usize'
        let buffer_len = usize::from_ne_bytes(read_buf::<{ size_of::<usize>() }>(iter)?);

        // deserialize duration as milliseconds as a 'u64'
        let millis = u64::from_ne_bytes(read_buf::<{ size_of::<u64>() }>(iter)?);

        // read frame buffer based on provided length
        let buffer = iter.take(buffer_len).collect();

        // construct frame
        Some(Self::new(buffer, Duration::from_millis(millis)))
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

    // consuming iterator for file contents
    let mut iter = bytes.into_iter();

    // number of frames
    let len =
        usize::from_ne_bytes(read_buf::<{ size_of::<usize>() }>(&mut iter).ok_or(Error::File)?);

    // read frame by frame
    let mut frames = Vec::with_capacity(len);
    for _ in 0..len {
        // deserialize frame
        let mut frame = AsciiFrame::from_bytes(&mut iter).ok_or(Error::File)?;

        // preprend home ascii escape sequence to each frame buffer
        frame.buffer.splice(0..0, "\x1b[H".bytes());
        frames.push(frame)
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

    // serialize frames beginning with number of frames
    let mut bytes = frames.len().to_ne_bytes().to_vec();
    bytes.extend(
        frames
            .into_iter()
            .flat_map(|f| f.into_bytes())
            .collect::<Vec<u8>>(),
    );

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
