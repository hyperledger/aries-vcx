pub const POSTGRES_ADDITIONAL_INITIALIZER: &str = "init_storagetype";
pub const DEFAULT_POSTGRES_PLUGIN_INITIALIZER: &str = "postgresstorage_init";

#[cfg(target_os = "macos")]
pub static DEFAULT_POSTGRES_PLUGIN_PATH: &str = "/usr/local/lib/libindystrgpostgres.dylib";
#[cfg(target_os = "linux")]
pub static DEFAULT_POSTGRES_PLUGIN_PATH: &str = "/usr/lib/libindystrgpostgres.so";
#[cfg(target_os = "windows")]
pub static DEFAULT_POSTGRES_PLUGIN_PATH: &str = "c:\\windows\\system32\\libindystrgpostgres.dll";