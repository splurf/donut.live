use std::time::Duration;

/// The initial HTTP response headers appended with the `ESC[2J` erase function
pub const INIT: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";

/// The delay between each frame
/// + 20.833333ms => ~48 FPS
pub const DELAY: Duration = Duration::from_nanos(20833333);

/// The characters required to make each donut frame
pub const CHARACTERS: [u8; 12] = [46, 44, 45, 126, 58, 59, 61, 33, 42, 35, 36, 64];
