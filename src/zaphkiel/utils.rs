use indicatif::{ProgressBar, ProgressStyle};

pub fn get_pb(len: u64, msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new(len);

    let pb_style = ProgressStyle::default_bar()
        .template(
            "{spinner:.green} [{elapsed}] {msg} [{wide_bar:.cyan/blue}] ({pos}/{len}|{percent}%) ({per_sec}|{eta})",
        )
        .unwrap()
        .progress_chars("#>-");
    pb.set_style(pb_style);
    pb.set_message(msg);
    pb.tick();

    pb
}
