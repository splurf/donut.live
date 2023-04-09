use {const_format::concatcp, std::time::Duration};

/** The header that each verfied curl request will receive */
const HEADER: &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n";

/** Sent to the client to clear their terminal */
pub const CLEAR: &'static [u8] = concatcp!(HEADER, "\x1b[2J").as_bytes();

/** Sent to the client to display the *greedy* message */
pub const GREED: &'static [u8] = concatcp!(HEADER, "you greedy fool\n").as_bytes();

/** The delay between each frame */
pub const DELAY: Duration = Duration::from_millis(16);

/** The characters required to make each donut frame */
pub const CHARACTERS: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];
