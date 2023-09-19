use std::time::Duration;

/** Sent to the client to *clear* their terminal */
pub const CLEAR: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";

/** The delay between each frame (this is roughly 48 fps) */
pub const DELAY: Duration = Duration::from_micros(20833);

/** The characters required to make each donut frame */
pub const CHARACTERS: [u8; 12] = [46, 44, 45, 126, 58, 59, 61, 33, 42, 35, 36, 64];
