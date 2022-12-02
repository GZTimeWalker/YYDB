use log::{LevelFilter, Metadata, Record};
use once_cell::sync::OnceCell;

static LOGGER: OnceCell<Logger> = OnceCell::new();

pub fn init() {
    log::set_logger(Logger::global()).unwrap();
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
    pub fn global() -> &'static Logger {
        LOGGER.get_or_init(|| {
            let profile = std::env::var("RUST_DEBUG");
            match profile {
                Ok(_) => Logger {
                    output: |_, message| {
                        println!("{}", message);
                    },
                },
                _ => Logger {
                    output: |level, message| {
                        crate::bridge::ffi::mysql_log_write(level as i32, message)
                    },
                },
            }
        })
    }

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
