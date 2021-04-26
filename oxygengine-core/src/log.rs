use std::sync::RwLock;

lazy_static! {
    static ref LOGGER: RwLock<Option<Box<dyn Logger>>> = RwLock::new(None);
}

#[derive(Debug, Copy, Clone)]
pub enum Log {
    Info,
    Warning,
    Error,
    Debug,
    #[cfg(feature = "profiler")]
    ProfileStart,
    #[cfg(feature = "profiler")]
    ProfileEnd,
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
            Log::Debug => eprintln!("[DEBUG] {}", message),
            #[cfg(feature = "profiler")]
            Log::ProfileStart => {}
            #[cfg(feature = "profiler")]
            Log::ProfileEnd => {}
        }
    }
}

pub fn logger_setup<L>(instance: L)
where
    L: Logger + 'static,
{
    if let Ok(mut logger) = LOGGER.write() {
        *logger = Some(Box::new(instance));
    }
}

pub fn logger_log(mode: Log, message: String) {
    if let Ok(mut logger) = LOGGER.write() {
        if let Some(ref mut logger) = *logger {
            logger.log(mode, message);
        }
    }
}

#[macro_export]
macro_rules! log {
    ($lvl:expr, $($arg:tt)*) => ({
        $crate::log::logger_log($lvl, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*)
        ));
    })
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        $crate::log::logger_log($crate::log::Log::Info, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*)
        ));
    })
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        $crate::log::logger_log($crate::log::Log::Warning, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*)
        ));
    })
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        $crate::log::logger_log($crate::log::Log::Error, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*)
        ));
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        $crate::log::logger_log($crate::log::Log::Debug, format!(
            "[{}: {} | {}]:\n{}",
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*)
        ));
    })
}

#[cfg(feature = "profiler")]
#[macro_export]
macro_rules! profile_scope {
    ($id:literal, $code:block) => {{
        $crate::log::logger_log($crate::log::Log::ProfileStart, $id.to_string());
        let result = { $code };
        $crate::log::logger_log($crate::log::Log::ProfileEnd, $id.to_string());
        result
    }};
}

#[cfg(not(feature = "profiler"))]
#[macro_export]
macro_rules! profile_scope {
    ($id:literal, $code:block) => {{
        $code
    }};
}
