use {
    super::{
        constants::{CLEAR, DELAY, GREED},
        error::*,
    },
    std::{
        io::Write,
        net::{Shutdown, TcpStream},
        sync::Mutex,
        thread::sleep,
    },
};

/** Continuously send each frame to the stream */
pub fn handle_stream(
    mut stream: impl Write,
    frames: &[Box<[u8; 1784]>],
    frame_index: &Mutex<usize>,
) -> Result<()> {
    stream.write_all(&CLEAR)?;

    //  aquire the lock and hold it until the connection is lost
    let mut frame_index_guard = frame_index.lock()?;

    //  place the frames into a cycle then advance the iterator to the last visited frame
    let mut frames_iter = frames.iter().enumerate().cycle();
    frames_iter.nth(*frame_index_guard);

    for (i, frame) in frames_iter {
        let result = stream.write_all(frame.as_slice());

        //  if the stream loses connection, set the frame index to the index of the current frame
        if result.is_err() {
            *frame_index_guard = i;
            result?
        }
        sleep(DELAY);
    }
    unreachable!("iterator is infinite")
}

/** Send the refused stream a goodbye message then shutdown the connection */
pub fn close_stream(mut stream: TcpStream) -> Result<()> {
    stream.write_all(&GREED)?;
    stream.shutdown(Shutdown::Both).map_err(Into::into)
}
