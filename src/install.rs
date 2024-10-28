use registry::{Hive, RegKey, Security};
use std::path::PathBuf;
use utfx::U16CString;

#[derive(Debug)]
pub enum Error {
    ErrorCreatingProjectKey,
    ErrorWritingProjectKey,
    FirefoxNotFound,
    InvalidJsonPath,
    ErrorWritingConfigData,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ErrorCreatingProjectKey => {
                write!(f, "Error creating project key in registry")
            }
            Error::ErrorWritingProjectKey => {
                write!(f, "Error writing project key to registry")
            }
            Error::FirefoxNotFound => {
                write!(f, "Firefox not found or installed")
            }
            Error::InvalidJsonPath => {
                write!(f, "JSON path cannot be written to registry")
            }
            Error::ErrorWritingConfigData => {
                write!(f, "Error writing config data")
            }
        }
    }
}

impl std::error::Error for Error {}

fn create_subkey_if_not_exist(rk: RegKey, name: &str) -> Result<RegKey, registry::key::Error> {
    match rk.open(name, Security::Read | Security::Write) {
        Ok(rk) => Ok(rk),
        Err(err) => match err {
            registry::key::Error::NotFound(..) => rk.create(name, Security::Read | Security::Write),
            _ => Err(err),
        },
    }
}

fn firefox_root_regkey() -> Result<RegKey, registry::key::Error> {
    Hive::CurrentUser.open(r"Software\\Mozilla\\", Security::Read | Security::Write)
}

fn create_nmh_regkey(rk: RegKey, program_name: &str) -> Result<RegKey, Error> {
    match create_subkey_if_not_exist(rk, "NativeMessagingHosts") {
        Ok(nmh) => create_subkey_if_not_exist(nmh, program_name)
            .map_err(|_| Error::ErrorCreatingProjectKey),
        Err(_) => Err(Error::ErrorCreatingProjectKey),
    }
}

/// Creates the registry keys for Firefox to detect the program as a NativeMessage host
pub fn firefox_registry_setup(json_location: &str, program_name: &str) -> Result<(), Error> {
    match firefox_root_regkey() {
        Ok(rk) => {
            let nmh = create_nmh_regkey(rk, program_name);
            match nmh {
                Ok(nmh) => match U16CString::from_str(json_location) {
                    Err(_) => Err(Error::InvalidJsonPath),
                    Ok(ws) => nmh
                        .set_value("", &registry::Data::String(ws))
                        .map_err(|_| Error::ErrorWritingProjectKey),
                },
                Err(_) => Err(Error::ErrorCreatingProjectKey),
            }
        }
        Err(_) => Err(Error::FirefoxNotFound),
    }
}

use serde_derive::*;
/// A native messaging manifest, as specified in [the Mozilla documentation](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests#native_messaging_manifests).
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Host {
    pub name: String,
    pub description: String,
    pub path: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub allowed_extensions: Vec<String>,
}

/// Creates the host JSON and the batch file it points to. Returns the path to the JSON.
pub fn nmh_files_setup(
    batch_contents: &str,
    host_path: PathBuf,
    script_path: PathBuf,
    extension_id: &str,
    host_name: &str,
    host_description: &str,
) -> Result<PathBuf, Error> {
    let script_result = std::fs::write(script_path.clone(), batch_contents)
        .map_err(|_| Error::ErrorWritingConfigData);
    if script_result.is_err() {
        return Err(Error::ErrorWritingConfigData);
    }
    let script_path = script_path.to_str().unwrap().to_string();
    let host = Host {
        name: host_name.to_string(),
        description: host_description.to_string(),
        path: script_path.clone(),
        _type: "stdio".to_string(),
        allowed_extensions: vec![extension_id.to_string()],
    };
    let host_file = serde_json::ser::to_string_pretty(&host).unwrap();
    let host_result =
        std::fs::write(host_path.clone(), host_file).map_err(|_| Error::ErrorWritingConfigData);
    // If both writing the JSON and batch script succeed, return the JSON path
    host_result.and(script_result).map(|_| host_path)
}
