use chrono::Local;
use log::{LevelFilter};
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
    Handle,
};

pub fn load_logger(level: LevelFilter) -> Option<Handle> {
    let level = level;
    let today = Local::now().format("%Y-%m-%d").to_string();
    let file_path = format!("log/{today}.log");
    let log_format = get_log_format(level);

    // Build a stderr logger.
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    // Logging to log file.
    let logfile = FileAppender::builder()
        // Pattern: https://docs.rs/log4rs/*/log4rs/encode/pattern/index.html
        .encoder(Box::new(PatternEncoder::new(log_format)))
        .build(file_path)
        .unwrap();

    // Log Trace level output to file where trace is the default level
    // and the programmatically specified level to stderr.
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    // Use this to change log levels at runtime.
    // This means you can change the default log level to trace
    // if you are trying to debug an issue and need more logs on then turn it off
    // once you are done.
    match log4rs::init_config(config) {
        Ok(handle) => {
            return Some(handle);
        }
        _ => {
            println!("Failed to initialize logger");

            return None;
        }
    }
}

fn get_log_format(level:LevelFilter) -> &'static str{
    let simple_format = "{h({l})} {d(%Y-%m-%d %H:%M:%S:%s)}:   {m}{n}\n";
    let full_format = "{h({l})} {d(%Y-%m-%d %H:%M:%S:%s)} [{f}:{L}] {M}:    {m}{n}\n";

    match level {
        LevelFilter::Off => "",
        LevelFilter::Trace | LevelFilter::Debug => full_format,
        _ => simple_format
    }
}
