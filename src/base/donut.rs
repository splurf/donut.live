use std::{f32::consts::TAU, time::Duration};

use indicatif::ProgressBar;

use super::{style, AsciiFrame};

/// The delay between each frame
/// - 20.833333ms => ~48 FPS
const DELAY: Duration = Duration::from_nanos(20833333);

/// The characters required to make each donut frame
const CHARACTERS: [u8; 12] = [46, 44, 45, 126, 58, 59, 61, 33, 42, 35, 36, 64];

/// Generate a single frame of the donut based on the given variables
fn gen_frame(
    a: &mut f32,
    b: &mut f32,
    i: &mut f32,
    j: &mut f32,
    z: &mut [f32; 1760],
    p: &mut [u8; 1760],
) -> Vec<u8> {
    while *j < TAU {
        while *i < TAU {
            let c = f32::sin(*i);
            let d = f32::cos(*j);
            let e = f32::sin(*a);
            let f = f32::sin(*j);
            let g = f32::cos(*a);
            let h = d + 2.0;
            let q = 1.0 / (c * h * e + f * g + 5.0);
            let l = f32::cos(*i);
            let m = f32::cos(*b);
            let n = f32::sin(*b);
            let t = c * h * g - f * e;

            let x = (40.0 + 30.0 * q * (l * h * m - t * n)) as i32;
            let y = (12.0 + 15.0 * q * (l * h * n + t * m)) as i32;
            let o = x + 80 * y;
            let n = (8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n)) as i32;

            if 22 > y && y > 0 && x > 0 && 80 > x && q > z[o as usize] {
                z[o as usize] = q;
                p[o as usize] = CHARACTERS[if n > 0 { n } else { 0 } as usize]
            }
            *i += 0.02
        }
        *i = 0.0;
        *j += 0.07
    }
    *a += 0.04;
    *b += 0.02;

    let frame = p
        .chunks_exact(80)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .join(&10);

    *p = [32; 1760];
    *z = [0.0; 1760];
    *j = 0.0;

    frame
}

/// *donut.c* refactored into rust
pub fn get_frames() -> Vec<AsciiFrame> {
    let mut a = 0.0;
    let mut b = 0.0;

    let mut i = 0.0;
    let mut j = 0.0;

    let mut z = [0.0; 1760];
    let mut p = [32; 1760];

    // Generate the original `donut` frames (559234 bytes)
    let pb = style(314, "Generating frames", false);
    let mut frames = [0; 314].map(|_| {
        let frame = gen_frame(&mut a, &mut b, &mut i, &mut j, &mut z, &mut p);
        pb.inc(1);
        frame
    });
    pb.finish();

    let pb = ProgressBar::new_spinner().with_message("Trimming frames");
    trim_frames(&mut frames); // 395012 bytes (~29% smaller)
    pb.finish_and_clear();

    frames.map(|buffer| AsciiFrame::new(buffer, DELAY)).to_vec()
}

/// Trim the *majority* of the unnecessary whitespace at the end of each line of every frame
/// TODO - Trim all redundant whitespace, without altering how the frames look
fn trim_frames(frames: &mut [Vec<u8>; 314]) {
    // Convenience method for returning the lines of each frame
    fn split(f: &[u8]) -> impl Iterator<Item = Vec<u8>> + '_ {
        f.split(|c| *c == 10).map(<[u8]>::to_vec)
    }

    // Determine the maximum length of non-ASCII of each line for every frame
    // TODO - Improve this or come up with a better algorithm
    let maxes = {
        let mut out = [[0; 314]; 22];

        for (j, chunk) in frames
            .iter()
            .flat_map(|f| {
                split(f)
                    .map(|u| {
                        u.iter()
                            .rposition(|c| CHARACTERS.contains(c))
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .chunks(22)
            .enumerate()
        {
            for (i, n) in chunk.iter().enumerate() {
                out[i][j] = *n
            }
        }
        out.into_iter()
            .map(|u| u.into_iter().max().unwrap_or_default())
            .collect::<Vec<_>>()
    };

    // Drain the ASCII of each line for every frame for their max length
    frames.iter_mut().for_each(|f| {
        *f = split(f.as_slice())
            .enumerate()
            .map(|(i, l)| l[0..maxes[i]].to_vec())
            .collect::<Vec<Vec<u8>>>()
            .join(&10);
    });
}
