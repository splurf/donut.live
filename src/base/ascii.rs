use artem::ConfigBuilder;
use bincode::{deserialize_from, serialize};
use gif::DecodeOptions;
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageDecoder};
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressIterator};
use log::info;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{
    fs::{read, File},
    io::{BufReader, Read, Seek, SeekFrom, Write},
    iter::repeat,
    path::Path,
    sync::Arc,
    thread::spawn,
    time::{Duration, Instant},
};
use zstd::{decode_all, zstd_safe::max_c_level};

use super::{donut, style, AsciiFrame, Config, Error, GifError, Result};

fn get_frames_count(mut opt: DecodeOptions, input: &mut BufReader<File>) -> Result<u64> {
    // adjust configuration
    opt.skip_frame_decoding(true);

    // decoder configured without LZW decoding
    let mut decoder = opt.read_info(input.by_ref())?;
    let mut count = 0;

    // begin decoding and counting frames
    let result = repeat(()).try_for_each(|_| {
        decoder
            .next_frame_info()?
            .ok_or(GifError::Eof.into())
            .map(|_| count += 1)
    });

    // ensure there are no issues
    if let Err(e) = result {
        if !matches!(e, Error::Gif(GifError::Eof)) {
            return Err(e);
        }
    }
    Ok(count)
}

fn get_ascii_frames(
    input: BufReader<File>,
    fps: Option<f32>,
    is_colored: bool,
    count: u64,
) -> Result<Vec<AsciiFrame>> {
    // // init decoder with new config
    let decoder = GifDecoder::new(input)?;

    // regulate then clamp dimensions
    let dims = {
        let (w, h) = decoder.dimensions();
        (h > 56).then_some(((w as f32 * (56.0 / h as f32)) as u32, 56))
    };

    // determine delay between each frame
    let delay = fps.map(|value| Duration::from_secs_f32(1.0 / value));

    // provide the color determinant
    let config = ConfigBuilder::new().color(is_colored).build();

    // decoded frames
    let frames = Arc::new(RwLock::new(Vec::<image::Frame>::new()));

    // progress bars
    let multi_pb = MultiProgress::new();
    let decoding_pb = multi_pb.insert(0, style(count, "Decoding frames", false));
    let ascii_pb = multi_pb.insert(1, style(count, "Converting frames into ASCII art", false));

    // begin decoding frames
    let frames_thread = frames.clone();
    let handle = spawn(move || {
        decoder
            .into_frames()
            .progress_with(decoding_pb)
            .try_for_each(|f| {
                frames_thread.write().push(f?);
                Ok::<_, Error>(())
            })
    });

    // convert frame whenever decoded frame(s) is/are available
    let mut ascii = Vec::<AsciiFrame>::with_capacity(count as usize);
    while ascii.len() < count as usize && !handle.is_finished() {
        frames.write().drain(..).try_for_each(|frame| {
            let frame_ascii = AsciiFrame::from_frame(frame.clone(), dims, delay, &config)?;
            ascii.push(frame_ascii);
            ascii_pb.inc(1);
            Ok::<_, Error>(())
        })?;
    }
    ascii_pb.finish();

    // check if decoding process short-circuited due to error
    handle.join().map_err(|_| Error::Sync)??;

    Ok(ascii)
}

fn get_frames_from_path(
    path: &Path,
    fps: Option<f32>,
    is_colored: bool,
) -> Result<Vec<AsciiFrame>> {
    // file reader
    let mut input = BufReader::new(File::open(path)?);

    let opt = DecodeOptions::new();

    let t = Instant::now();

    // determine number of frames
    let count = get_frames_count(opt.clone(), input.by_ref())?;
    info!("ELAPSED: {:?}", t.elapsed());
    info!("Number of frames: {}", count);

    // seek back to beginning of file
    input.seek(SeekFrom::Start(0))?;

    // convert frames into ASCII
    get_ascii_frames(input, fps, is_colored, count)
}

fn read_file(file_name: &str) -> Result<Vec<AsciiFrame>> {
    // read contents of file
    info!("Looking for {:?} file", file_name);
    let input = read(file_name)?;

    // decompress file contents
    let read = style(input.len() as u64, "Decompressing", true).wrap_read(input.as_slice());
    let decompressed = decode_all(read)?;

    // deserialize decompressed data
    let read =
        style(decompressed.len() as u64, "Deserializing", true).wrap_read(decompressed.as_slice());
    deserialize_from(read).map_err(Into::into)
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
        donut::get_frames() // default
    };

    // serialize to bytes
    info!("Serializing data");
    let serialized = serialize(frames.as_slice())?;

    // write to file while compressing serialization
    let input = File::create(file_name)?;

    // write compressed data through encoder in chunks
    let mut encoder = zstd::Encoder::new(input, max_c_level())?;
    let mut write = style(
        serialized.len() as u64,
        "Compressing data and writing to file",
        true,
    )
    .wrap_write(encoder.by_ref());
    serialized
        .chunks(128)
        .try_for_each(|chunk| write.write_all(chunk))?;
    encoder.finish()?;

    // return the generated frames
    Ok(frames)
}

pub fn get_frames(cfg: &Config) -> Result<Vec<AsciiFrame>> {
    // generate and write frames to file if they don't already exist
    let mut frames = read_file(cfg.file_name()).or_else(|_| {
        // save to file, while returning original result
        write_file(cfg.gif(), cfg.fps(), cfg.is_colored(), cfg.file_name())
    })?;

    // number of frames
    let frames_count = frames.len();

    // preprend home ascii escape sequence to each frame buffer
    frames
        .par_iter_mut()
        .progress_with(style(frames_count as u64, "Finishing frames", false))
        .for_each(|f| f.prepend_home_esc());
    Ok(frames)
}
