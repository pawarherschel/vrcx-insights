#[macro_export]
macro_rules! time_it {
    ($comment:literal => $stmt:stmt) => {{
        time_it!(concat!($comment, "") => {$stmt})
    }};
    (at once | $comment:literal => $stmt:stmt) => {{
        time_it!(at once | concat!($comment, "") => {$stmt})
    }};
    ($comment:expr => $stmt:stmt) => {{
        use std::io::Write;
        print!("{}", $comment);
        let _ = std::io::stdout().flush();
        let start = std::time::Instant::now();
        let result = { $stmt };
        let duration = start.elapsed();
        println!(" => {:?}", duration);
        result
    }};
    (at once | $comment:expr => $stmt:stmt) => {{
        use std::io::Write;
        let start = std::time::Instant::now();
        let result = { $stmt };
        let duration = start.elapsed();
        println!("{} => {:?}", $comment, duration);
        result
    }};
}
#[macro_export]
macro_rules! debug {
    ($val:expr) => {
        #[cfg(debug_assertions)]
        {
            dbg!($val)
        }
    };
    ($($val:expr),+ $(,)?) => {
        #[cfg(debug_assertions)]
        {
            dbg!($($val),+)
        }
    };
}
