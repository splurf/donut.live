use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};

pub fn style(count: u64, msg: &'static str, is_bytes: bool) -> ProgressBar {
    let (pos, len) = if is_bytes {
        ("binary_bytes", "binary_total_bytes")
    } else {
        ("pos", "len")
    };
    let style = unsafe {
        ProgressStyle::with_template(
            format!("[{{eta_precise}}] {{bar:32}} {{{pos}:>6}}/{{{len}:6}} {{msg}}").as_str(),
        )
        .unwrap_unchecked()
    };

    ProgressBar::new(count)
        .with_message(msg)
        .with_style(style)
        .with_finish(ProgressFinish::Abandon)
}
