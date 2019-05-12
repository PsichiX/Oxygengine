use std::sync::Mutex;

lazy_static! {
    static ref LOGGER: Mutex<Option<Box<dyn Logger>>> = Mutex::new(None);
}

#[derive(Debug, Copy, Clone)]
pub enum Log {
    Info,
    Warning,
    Error,
}

pub trait Logger: Send + Sync {
    fn log(&mut self, mode: Log, message: String);
}

pub struct DefaultLogger;

impl Logger for DefaultLogger {
    fn log(&mut self, mode: Log, message: String) {
        match mode {
            Log::Info => println!("[INFO] {}", message),
            Log::Warning => eprintln!("[WARNING] {}", message),
            Log::Error => eprintln!("[ERROR] {}", message),
        }
    }
}

pub fn logger_setup<L>(instance: L)
where
    L: Logger + 'static,
{
    if let Ok(mut logger) = LOGGER.lock() {
        *logger = Some(Box::new(instance));
    }
}

pub fn logger_log(mode: Log, message: String) {
    if let Ok(mut logger) = LOGGER.lock() {
        if let Some(ref mut logger) = *logger {
            logger.log(mode, message);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)+) => ({
        $crate::log::logger_log($lvl, format!(
            "[{}: {} | {}]: {}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)+)
        ))
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => (log!($crate::log::Log::Info, $($arg)*))
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => (log!($crate::log::Log::Warning, $($arg)*))
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => (log!($crate::log::Log::Error, $($arg)*))
}
