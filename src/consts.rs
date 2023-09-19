<<<<<<< HEAD
use std::time::Duration;

/** Sent to the client to *clear* their terminal */
pub const CLEAR: &'static [u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n\x1b[2J";
=======
use {const_format::concatcp, std::time::Duration};

/** The header that each verfied curl request will receive */
const HEADER: &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n";

/** Sent to the client to *clear* their terminal */
pub const CLEAR: &'static [u8] = concatcp!(HEADER, "\x1b[2J").as_bytes();

/** Sent to the client to display the *greedy* message */
pub const GREED: &'static [u8] = concatcp!(HEADER, "you greedy fool\n").as_bytes();
>>>>>>> e13826e13ffea178880ea30ab20dd6cad2efd513

/** The delay between each frame (this is roughly 48 fps) */
pub const DELAY: Duration = Duration::from_micros(20833);

/** The characters required to make each donut frame */
<<<<<<< HEAD
pub const CHARACTERS: [u8; 12] = [46, 44, 45, 126, 58, 59, 61, 33, 42, 35, 36, 64];
=======
pub const CHARACTERS: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];
>>>>>>> e13826e13ffea178880ea30ab20dd6cad2efd513
