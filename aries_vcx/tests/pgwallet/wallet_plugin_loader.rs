use std::ffi::CString;

use libc::c_char;

use aries_vcx::indy::ErrorCode;

use crate::pgwallet::constants::POSTGRES_ADDITIONAL_INITIALIZER;
use crate::pgwallet::constants::DEFAULT_POSTGRES_PLUGIN_PATH;
use crate::pgwallet::constants::DEFAULT_POSTGRES_PLUGIN_INITIALIZER;

#[cfg(all(unix, test))]
pub fn load_lib(library: &str) -> libloading::Result<libloading::Library> {
    libloading::os::unix::Library::open(Some(library), ::libc::RTLD_NOW | ::libc::RTLD_NODELETE)
        .map(libloading::Library::from)
}

#[cfg(any(not(unix), not(test)))]
pub fn load_lib(library: &str) -> libloading::Result<libloading::Library> {
    libloading::Library::new(library)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PluginInitConfig {
    pub storage_type: String,
    // Optional to override default library path. Default value is determined based on value of
    // xtype and OS
    pub plugin_library_path: Option<String>,
    // Optional to override default storage initialization function. Default value is  determined
    // based on value of xtype and OS
    pub plugin_init_function: Option<String>,
    // Wallet storage config for agents wallets
    pub config: String,
    // Wallet storage credentials for agents wallets
    pub credentials: String,
}

pub fn init_wallet_plugin(plugin_init_config: &PluginInitConfig) -> Result<(), String> {
    debug!("init_wallet_plugin >>> wallet_storage_config: {}", serde_json::to_string(plugin_init_config).unwrap());

    let plugin_library_path = get_plugin_library_path(&plugin_init_config.storage_type, &plugin_init_config.plugin_library_path)?;
    let plugin_init_function = get_plugin_init_function(&plugin_init_config.storage_type, &plugin_init_config.plugin_init_function)?;
    let lib = load_storage_library(&plugin_library_path, &plugin_init_function)?;
    if plugin_init_config.storage_type == "postgres_storage" {
        finish_loading_postgres(lib, &plugin_init_config.config, &plugin_init_config.credentials)?;
    }
    debug!("Successfully loaded wallet plugin.");
    Ok(())
}

fn load_storage_library(library: &str, initializer: &str) -> Result<libloading::Library, String> {
    debug!("Loading storage plugin '{:}' as a dynamic library.", library);
    match load_lib(library) {
        Ok(lib) => {
            unsafe {
                debug!("Storage library '{:}' loaded. Resolving its init function '{:}'.", library, initializer);
                let init_func: libloading::Symbol<unsafe extern fn() -> ErrorCode> = lib.get(initializer.as_bytes()).unwrap();
                debug!("Initializing library '{:}' by calling function '{:}'.", library, initializer);
                match init_func() {
                    ErrorCode::Success => debug!("Basic initialization for library '{:}' succeeded.", library),
                    err => return Err(format!("Failed to resolve init function '{:}' for storage library '{:}'. Details {:?}.", initializer, library, err))
                };
                Ok(lib)
            }
        }
        Err(err) => Err(format!("Storage library {:} failed to load. Details: {:?}", library, err))
    }
}

fn finish_loading_postgres(storage_lib: libloading::Library, storage_config: &str, storage_credentials: &str) -> Result<(), String> {
    unsafe {
        debug!("Finishing initialization for postgres wallet plugin.");
        let init_storage_func: libloading::Symbol<unsafe extern fn(config: *const c_char, credentials: *const c_char) -> ErrorCode> = storage_lib.get(POSTGRES_ADDITIONAL_INITIALIZER.as_bytes()).unwrap();
        let init_config = CString::new(storage_config).expect("CString::new failed");
        let init_credentials = CString::new(storage_credentials).expect("CString::new failed");
        match init_storage_func(init_config.as_ptr(), init_credentials.as_ptr()) {
            ErrorCode::Success => {
                debug!("Successfully completed postgres library initialization.");
            }
            err => return Err(format!("Failed to complete postgres library initialization. Details {:?}.", err))
        }
    }
    Ok(())
}

fn get_plugin_library_path(storage_type: &str, plugin_library_path: &Option<String>) -> Result<String, String> {
    if storage_type == "postgres_storage" {
        Ok(plugin_library_path.clone().unwrap_or(DEFAULT_POSTGRES_PLUGIN_PATH.into()))
    } else {
        plugin_library_path.clone()
            .ok_or(format!("You have to specify 'storage.plugin_library_path' in config because storage of type {} does not have known default path.", storage_type))
    }
}

fn get_plugin_init_function(storage_type: &str, plugin_init_function: &Option<String>) -> Result<String, String> {
    if storage_type == "postgres_storage" {
        Ok(plugin_init_function.clone().unwrap_or(DEFAULT_POSTGRES_PLUGIN_INITIALIZER.into()))
    } else {
        plugin_init_function.clone()
            .ok_or(format!("You have to specify 'storage.plugin_init_function' in con_load_libfig because storage of type {} does not have known default path.", storage_type))
    }
}
