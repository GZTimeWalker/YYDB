use log::{LevelFilter, Metadata, Record};

lazy_static! {
    static ref LOGGER: Logger = {
        let profile = std::env::var("RUST_DEBUG");
        match profile {
            Ok(_) => Logger {
                output: |_, message| {
                    println!("{message}");
                },
            },
            _ => Logger {
                output: |level, message| crate::bridge::ffi::mysql_log_write(level as i32, message),
            },
        }
    };
}

/// Init the logger of YYDB.
pub(crate) fn init() {
    // an error will be returned if the logger was
    // already initialized. this situation is expected
    // when we reinstall the plugin. 
    log::set_logger(&*LOGGER).ok();

    log::set_max_level(match option_env!("LOG_LEVEL") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });

    info!("Current log level: {}", log::max_level());
    info!("Logger Initialized.");
}

struct Logger {
    output: fn(log::Level, &str),
}

impl Logger {
    fn get_formatted_message(&self, record: &Record) -> String {
        match record.level() {
            log::Level::Error => format!(
                "[Err] {}@{}: {}",
                record.file_static().unwrap_or(""),
                record.line().unwrap_or(0),
                record.args()
            ),
            log::Level::Warn => format!("[Wrn] {}", record.args()),
            log::Level::Info => format!("[Inf] {}", record.args()),
            log::Level::Debug => format!("[Dug] {}", record.args()),
            log::Level::Trace => format!("[Vrb] {}", record.args()),
        }
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            (self.output)(record.level(), &self.get_formatted_message(record));
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        init();
        trace!("Hello, world!");
        debug!("Hello, world!");
        info!("Hello, world!");
        warn!("Hello, world!");
        error!("Hello, world!");
    }
}
