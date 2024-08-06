use image::{imageops::FilterType, DynamicImage};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{GifError, Result};

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

    pub fn from_frame(
        frame: image::Frame,
        dims: Option<(u32, u32)>,
        delay: Option<Duration>,
        config: &artem::config::Config,
    ) -> Result<Self> {
        // use specified delay or delay of the current frame
        let delay = delay.unwrap_or(frame.delay().into());

        // ensure frame delays are consistent
        if delay.is_zero() {
            return Err(GifError::Delay.into());
        }

        // represent image buffer as dynamic image
        let mut img = DynamicImage::from(frame.into_buffer());

        // resize if needed
        if let Some((w, h)) = dims {
            // resize image dimensions based on provided dimensions
            img = img.resize_exact(w, h, FilterType::Nearest);
        }

        // convert image into ASCII art
        let s = artem::convert(img, config);

        // buffer and delay data only
        Ok(Self::new(s.into_bytes(), delay))
    }

    pub const fn delay(&self) -> Duration {
        self.delay
    }

    pub fn prepend_home_esc(&mut self) {
        self.buffer.splice(0..0, "\x1b[H".bytes());
    }
}

impl AsRef<[u8]> for AsciiFrame {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}
