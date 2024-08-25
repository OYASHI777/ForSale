// use env_logger::Builder;
// use log::LevelFilter;
// use std::io::Write;

// pub fn init_logger(level: LevelFilter) {
//     let mut builder = Builder::new();
//
//     // Set the logging level
//     builder.filter(None, level);
//
//     // Set the format of the log output
//     builder.format(|buf, record| {
//         writeln!(
//             buf,
//             "{} [{}] - {}:{} - {}",
//             chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
//             record.level(),
//             record.file().unwrap_or("unknown"),
//             record.line().unwrap_or(0),
//             record.args()
//         )
//     });
//
//     // Initialize the logger
//     builder.init();
// }
use env_logger::Builder;
use log::LevelFilter;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn init_logger(level: LevelFilter, filename: &str) {
    // Ensure the logs directory exists
    let log_dir = Path::new("./logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir).expect("Failed to create logs directory");
    }

    // Create or open the log file
    let log_file_path = log_dir.join(filename);
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true) // Append to the file if it already exists
        .open(log_file_path)
        .expect("Failed to open log file");

    let mut builder = Builder::new();

    // Set the logging level
    builder.filter(None, level);

    // Set the format of the log output
    builder.format(move |buf, record| {
        writeln!(
            buf,
            "{} [{}] - {}:{} - {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    });

    // Direct logs to the file instead of stdout
    builder.target(env_logger::Target::Pipe(Box::new(log_file)));

    // Initialize the logger
    builder.init();
}
