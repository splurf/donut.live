use std::f32::consts::TAU;

/**
 * donut.c rewritten except it returns the *stdout* as a vector
 */
pub fn donuts() -> Vec<Vec<u8>> {
    const CHARACTERS: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

    let mut a = 0.0;
    let mut b = 0.0;

    let mut i = 0.0;
    let mut j = 0.0;

    let mut z = [0.0; 1760];
    let mut p = [' '; 1760];

    (0..314)
        .map(|_| {
            while j < TAU {
                while i < TAU {
                    let c = f32::sin(i);
                    let d = f32::cos(j);
                    let e = f32::sin(a);
                    let f = f32::sin(j);
                    let g = f32::cos(a);
                    let h = d + 2.0;
                    let q = 1.0 / (c * h * e + f * g + 5.0);
                    let l = f32::cos(i);
                    let m = f32::cos(b);
                    let n = f32::sin(b);
                    let t = c * h * g - f * e;

                    let x = (40.0 + 30.0 * q * (l * h * m - t * n)) as i32;
                    let y = (12.0 + 15.0 * q * (l * h * n + t * m)) as i32;
                    let o = x + 80 * y;
                    let n =
                        (8.0 * ((f * e - c * d * g) * m - c * d * e - f * g - l * d * n)) as i32;

                    if 22 > y && y > 0 && x > 0 && 80 > x && q > z[o as usize] {
                        z[o as usize] = q;
                        p[o as usize] = CHARACTERS[if n > 0 { n } else { 0 } as usize]
                    }
                    i += 0.02
                }
                i = 0.0;
                j += 0.07
            }
            a += 0.04;
            b += 0.02;

            let frames = format!(
                "\x1b[H{}",
                p.chunks(80)
                    .map(|l| l.iter().collect())
                    .collect::<Vec<String>>()
                    .join("\n")
            )
            .as_bytes()
            .to_vec();

            p = [' '; 1760];
            z = [0.0; 1760];
            j = 0.0;

            frames
        })
        .collect()
}
