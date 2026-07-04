use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::{cell::RefCell, collections::BTreeMap};

use serde::{Deserialize, Serialize};

const DEFAULT_ENDPOINT: &str = "http://localhost:6800/jsonrpc";
const DEFAULT_POLLING_INTERVAL_SECONDS: u16 = 2;
#[cfg(not(test))]
const KEYRING_SERVICE: &str = "cottid";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonMode {
    Managed,
    External,
}

impl DaemonMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Managed => "Managed",
            Self::External => "External",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    endpoint: String,
    auth: RpcAuth,
    polling_interval_seconds: u16,
    websocket_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            auth: RpcAuth::NoSecret,
            polling_interval_seconds: DEFAULT_POLLING_INTERVAL_SECONDS,
            websocket_enabled: true,
        }
    }
}

impl Settings {
    pub fn new_without_secret(
        endpoint: impl Into<String>,
        polling_interval_seconds: u16,
    ) -> Result<Self, EndpointValidationError> {
        Self::new(endpoint, RpcAuth::NoSecret, polling_interval_seconds, true)
    }

    fn new(
        endpoint: impl Into<String>,
        auth: RpcAuth,
        polling_interval_seconds: u16,
        websocket_enabled: bool,
    ) -> Result<Self, EndpointValidationError> {
        let endpoint = endpoint.into();
        Self::validate_endpoint(&endpoint)?;

        Ok(Self {
            endpoint: endpoint.trim().to_owned(),
            auth,
            polling_interval_seconds: polling_interval_seconds.max(1),
            websocket_enabled,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn auth(&self) -> &RpcAuth {
        &self.auth
    }

    pub fn polling_interval_seconds(&self) -> u16 {
        self.polling_interval_seconds
    }

    pub fn websocket_enabled(&self) -> bool {
        self.websocket_enabled
    }

    pub fn validate_endpoint(endpoint: &str) -> Result<(), EndpointValidationError> {
        validate_endpoint(endpoint)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStorage {
    None,
    Keyring,
    PlaintextFallback,
    SessionOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }

    pub fn from_config_value(value: &str) -> Option<Self> {
        match value {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedConfig {
    daemon_mode: DaemonMode,
    settings: Settings,
    selected_filter: String,
    auth_storage: AuthStorage,
    theme_preference: ThemePreference,
    confirm_destructive_actions: bool,
    notify_download_outcomes: bool,
    new_download_directory: String,
    new_download_output_filename: String,
    new_download_max_download_limit: String,
    new_download_max_upload_limit: String,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        Self {
            daemon_mode: DaemonMode::Managed,
            settings: Settings::default(),
            selected_filter: "active".to_owned(),
            auth_storage: AuthStorage::None,
            theme_preference: ThemePreference::System,
            confirm_destructive_actions: true,
            notify_download_outcomes: false,
            new_download_directory: String::new(),
            new_download_output_filename: String::new(),
            new_download_max_download_limit: String::new(),
            new_download_max_upload_limit: String::new(),
        }
    }
}

impl PersistedConfig {
    pub fn with_auth_storage(
        settings: Settings,
        selected_filter: impl Into<String>,
        auth_storage: AuthStorage,
    ) -> Self {
        Self::with_auth_storage_and_theme(
            settings,
            selected_filter,
            auth_storage,
            ThemePreference::System,
        )
    }

    pub fn with_auth_storage_and_theme(
        settings: Settings,
        selected_filter: impl Into<String>,
        auth_storage: AuthStorage,
        theme_preference: ThemePreference,
    ) -> Self {
        Self {
            daemon_mode: DaemonMode::External,
            settings,
            selected_filter: selected_filter.into(),
            auth_storage,
            theme_preference,
            confirm_destructive_actions: true,
            notify_download_outcomes: false,
            new_download_directory: String::new(),
            new_download_output_filename: String::new(),
            new_download_max_download_limit: String::new(),
            new_download_max_upload_limit: String::new(),
        }
    }

    pub fn with_daemon_mode(mut self, daemon_mode: DaemonMode) -> Self {
        self.daemon_mode = daemon_mode;
        self
    }

    fn with_theme_preference(mut self, theme_preference: ThemePreference) -> Self {
        self.theme_preference = theme_preference;
        self
    }

    pub fn with_ui_preferences(
        mut self,
        confirm_destructive_actions: bool,
        notify_download_outcomes: bool,
    ) -> Self {
        self.confirm_destructive_actions = confirm_destructive_actions;
        self.notify_download_outcomes = notify_download_outcomes;
        self
    }

    pub fn with_new_download_directory(mut self, directory: impl Into<String>) -> Self {
        self.new_download_directory = directory.into();
        self
    }

    pub fn with_new_download_defaults(
        mut self,
        output_filename: impl Into<String>,
        max_download_limit: impl Into<String>,
        max_upload_limit: impl Into<String>,
    ) -> Self {
        self.new_download_output_filename = output_filename.into();
        self.new_download_max_download_limit = max_download_limit.into();
        self.new_download_max_upload_limit = max_upload_limit.into();
        self
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn daemon_mode(&self) -> DaemonMode {
        self.daemon_mode
    }

    pub fn selected_filter(&self) -> &str {
        &self.selected_filter
    }

    pub fn auth_storage(&self) -> AuthStorage {
        self.auth_storage
    }

    pub fn theme_preference(&self) -> ThemePreference {
        self.theme_preference
    }

    pub fn confirm_destructive_actions(&self) -> bool {
        self.confirm_destructive_actions
    }

    pub fn notify_download_outcomes(&self) -> bool {
        self.notify_download_outcomes
    }

    pub fn new_download_directory(&self) -> &str {
        &self.new_download_directory
    }

    pub fn new_download_output_filename(&self) -> &str {
        &self.new_download_output_filename
    }

    pub fn new_download_max_download_limit(&self) -> &str {
        &self.new_download_max_download_limit
    }

    pub fn new_download_max_upload_limit(&self) -> &str {
        &self.new_download_max_upload_limit
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLoad {
    config: PersistedConfig,
    feedback: Option<String>,
}

impl ConfigLoad {
    #[cfg(test)]
    pub fn config(&self) -> &PersistedConfig {
        &self.config
    }

    pub fn into_config(self) -> PersistedConfig {
        self.config
    }

    pub fn feedback(&self) -> Option<&str> {
        self.feedback.as_deref()
    }
}

#[derive(Debug)]
pub struct ConfigSaveError {
    kind: ConfigSaveErrorKind,
}

#[derive(Debug)]
enum ConfigSaveErrorKind {
    Io(io::Error),
    Serialize(toml::ser::Error),
    TokenStore(TokenStoreError),
}

impl ConfigSaveError {
    pub fn message(&self) -> &'static str {
        match self.kind {
            ConfigSaveErrorKind::TokenStore(_) => {
                "Token could not be stored securely. Choose plaintext fallback or keep it session-only."
            }
            ConfigSaveErrorKind::Io(_) | ConfigSaveErrorKind::Serialize(_) => {
                "Config could not be saved."
            }
        }
    }

    pub fn is_token_store_error(&self) -> bool {
        matches!(self.kind, ConfigSaveErrorKind::TokenStore(_))
    }
}

impl fmt::Display for ConfigSaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ConfigSaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ConfigSaveErrorKind::Io(source) => Some(source),
            ConfigSaveErrorKind::Serialize(source) => Some(source),
            ConfigSaveErrorKind::TokenStore(source) => Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStoreError {
    message: String,
}

impl TokenStoreError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for TokenStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TokenStoreError {}

pub trait TokenStore {
    fn load(&self, endpoint: &str) -> Result<Option<Secret>, TokenStoreError>;
    fn save(&self, endpoint: &str, secret: &Secret) -> Result<(), TokenStoreError>;
    fn delete(&self, endpoint: &str) -> Result<(), TokenStoreError>;
}

#[derive(Debug, Clone, Copy)]
pub struct SystemTokenStore;

#[cfg(test)]
thread_local! {
    static TEST_SYSTEM_TOKENS: RefCell<BTreeMap<String, Secret>> = const { RefCell::new(BTreeMap::new()) };
}

impl TokenStore for SystemTokenStore {
    fn load(&self, endpoint: &str) -> Result<Option<Secret>, TokenStoreError> {
        #[cfg(test)]
        {
            Ok(TEST_SYSTEM_TOKENS.with(|tokens| tokens.borrow().get(endpoint).cloned()))
        }

        #[cfg(not(test))]
        {
            let entry =
                keyring::Entry::new(KEYRING_SERVICE, endpoint).map_err(token_store_error)?;
            match entry.get_password() {
                Ok(secret) => Ok(Some(Secret::session(secret))),
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => Err(token_store_error(error)),
            }
        }
    }

    fn save(&self, endpoint: &str, secret: &Secret) -> Result<(), TokenStoreError> {
        #[cfg(test)]
        {
            TEST_SYSTEM_TOKENS.with(|tokens| {
                tokens
                    .borrow_mut()
                    .insert(endpoint.to_owned(), secret.clone());
            });
            Ok(())
        }

        #[cfg(not(test))]
        {
            let entry =
                keyring::Entry::new(KEYRING_SERVICE, endpoint).map_err(token_store_error)?;
            entry
                .set_password(secret.expose_for_session())
                .map_err(token_store_error)
        }
    }

    fn delete(&self, endpoint: &str) -> Result<(), TokenStoreError> {
        #[cfg(test)]
        {
            TEST_SYSTEM_TOKENS.with(|tokens| {
                tokens.borrow_mut().remove(endpoint);
            });
            Ok(())
        }

        #[cfg(not(test))]
        {
            let entry =
                keyring::Entry::new(KEYRING_SERVICE, endpoint).map_err(token_store_error)?;
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                Err(error) => Err(token_store_error(error)),
            }
        }
    }
}

#[cfg(not(test))]
fn token_store_error(error: keyring::Error) -> TokenStoreError {
    TokenStoreError::new(error.to_string())
}

pub fn default_config_path() -> PathBuf {
    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return xdg_config_home.join("cottid").join("config");
    }

    if let Some(home) = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return home.join(".config").join("cottid").join("config");
    }

    PathBuf::from("cottid").join("config")
}

pub fn load_config(path: &Path) -> ConfigLoad {
    load_config_with_token_store(path, &SystemTokenStore)
}

pub fn load_config_with_token_store(path: &Path, token_store: &dyn TokenStore) -> ConfigLoad {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return ConfigLoad {
                config: PersistedConfig::default(),
                feedback: None,
            };
        }
        Err(_) => {
            return ConfigLoad {
                config: PersistedConfig::default(),
                feedback: Some("Config could not be read; using defaults.".to_owned()),
            };
        }
    };

    match parse_config(&contents, token_store) {
        Ok(load) => load,
        Err(()) => ConfigLoad {
            config: PersistedConfig::default(),
            feedback: Some("Config was invalid; using defaults.".to_owned()),
        },
    }
}

pub fn save_config_with_token_store(
    path: &Path,
    config: &PersistedConfig,
    previous_endpoint: Option<&str>,
    token_store: &dyn TokenStore,
) -> Result<(), ConfigSaveError> {
    persist_secret(config, token_store)?;
    write_config_file(path, config)?;

    if let Some(previous_endpoint) = previous_endpoint
        && previous_endpoint != config.settings().endpoint()
    {
        token_store
            .delete(previous_endpoint)
            .map_err(|source| ConfigSaveError {
                kind: ConfigSaveErrorKind::TokenStore(source),
            })?;
    }

    Ok(())
}

pub fn save_config_without_token_store(
    path: &Path,
    config: &PersistedConfig,
) -> Result<(), ConfigSaveError> {
    write_config_file(path, config)
}

fn write_config_file(path: &Path, config: &PersistedConfig) -> Result<(), ConfigSaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigSaveError {
            kind: ConfigSaveErrorKind::Io(source),
        })?;
    }

    fs::write(
        path,
        serialize_config(config).map_err(|source| ConfigSaveError {
            kind: ConfigSaveErrorKind::Serialize(source),
        })?,
    )
    .map_err(|source| ConfigSaveError {
        kind: ConfigSaveErrorKind::Io(source),
    })
}

fn persist_secret(
    config: &PersistedConfig,
    token_store: &dyn TokenStore,
) -> Result<(), ConfigSaveError> {
    match (config.auth_storage(), config.settings().auth()) {
        (AuthStorage::Keyring, RpcAuth::SessionSecret(secret)) => token_store
            .save(config.settings().endpoint(), secret)
            .map_err(|source| ConfigSaveError {
                kind: ConfigSaveErrorKind::TokenStore(source),
            }),
        (AuthStorage::None, _) | (_, RpcAuth::NoSecret) => {
            token_store
                .delete(config.settings().endpoint())
                .map_err(|source| ConfigSaveError {
                    kind: ConfigSaveErrorKind::TokenStore(source),
                })?;
            Ok(())
        }
        (AuthStorage::PlaintextFallback | AuthStorage::SessionOnly, RpcAuth::SessionSecret(_)) => {
            Ok(())
        }
    }
}

fn parse_config(contents: &str, token_store: &dyn TokenStore) -> Result<ConfigLoad, ()> {
    match toml::from_str::<TomlConfig>(contents) {
        Ok(toml_config) => config_from_toml(toml_config, token_store),
        Err(_) => config_from_legacy(contents),
    }
}

fn config_from_toml(config: TomlConfig, token_store: &dyn TokenStore) -> Result<ConfigLoad, ()> {
    let connection = config.connection.ok_or(())?;
    let daemon_mode = config
        .daemon
        .and_then(|daemon| daemon.mode)
        .map(Into::into)
        .unwrap_or(DaemonMode::External);
    let endpoint = connection
        .endpoint
        .unwrap_or_else(|| DEFAULT_ENDPOINT.to_owned());
    let polling_interval_seconds = connection
        .polling_interval_seconds
        .unwrap_or(DEFAULT_POLLING_INTERVAL_SECONDS);
    let websocket_enabled = connection.websocket_enabled.unwrap_or(true);
    let auth = config.auth.unwrap_or_default();
    let auth_storage = auth.storage.unwrap_or(TomlAuthStorage::None).into();

    let mut feedback = None;
    let rpc_auth = match auth_storage {
        AuthStorage::None | AuthStorage::SessionOnly => RpcAuth::NoSecret,
        AuthStorage::PlaintextFallback => match auth.plaintext_token {
            Some(token) if !token.is_empty() => RpcAuth::SessionSecret(Secret::session(token)),
            _ => {
                feedback = Some("Stored token could not be loaded; enter it again.".to_owned());
                RpcAuth::NoSecret
            }
        },
        AuthStorage::Keyring => match token_store.load(&endpoint) {
            Ok(Some(secret)) => RpcAuth::SessionSecret(secret),
            Ok(None) | Err(_) => {
                feedback = Some("Stored token could not be loaded; enter it again.".to_owned());
                RpcAuth::NoSecret
            }
        },
    };

    let settings = Settings::new(
        endpoint,
        rpc_auth,
        polling_interval_seconds,
        websocket_enabled,
    )
    .map_err(|_| ())?;
    let selected_filter = config
        .ui
        .as_ref()
        .and_then(|ui| ui.selected_filter.clone())
        .unwrap_or_else(|| "active".to_owned());
    let theme_preference = config
        .ui
        .as_ref()
        .and_then(|ui| ui.theme)
        .map(Into::into)
        .unwrap_or(ThemePreference::System);
    let confirm_destructive_actions = config
        .ui
        .as_ref()
        .and_then(|ui| ui.confirm_destructive_actions)
        .unwrap_or(true);
    let notify_download_outcomes = config
        .ui
        .as_ref()
        .and_then(|ui| ui.notify_download_outcomes)
        .unwrap_or(false);
    let new_download_directory = config
        .new_download
        .as_ref()
        .and_then(|new_download| new_download.directory.clone())
        .unwrap_or_default();
    let new_download_output_filename = config
        .new_download
        .as_ref()
        .and_then(|new_download| new_download.output_filename.clone())
        .unwrap_or_default();
    let new_download_max_download_limit = config
        .new_download
        .as_ref()
        .and_then(|new_download| new_download.max_download_limit.clone())
        .unwrap_or_default();
    let new_download_max_upload_limit = config
        .new_download
        .as_ref()
        .and_then(|new_download| new_download.max_upload_limit.clone())
        .unwrap_or_default();

    Ok(ConfigLoad {
        config: PersistedConfig::with_auth_storage_and_theme(
            settings,
            selected_filter,
            auth_storage,
            theme_preference,
        )
        .with_daemon_mode(daemon_mode)
        .with_ui_preferences(confirm_destructive_actions, notify_download_outcomes)
        .with_new_download_directory(new_download_directory)
        .with_new_download_defaults(
            new_download_output_filename,
            new_download_max_download_limit,
            new_download_max_upload_limit,
        ),
        feedback,
    })
}

fn config_from_legacy(contents: &str) -> Result<ConfigLoad, ()> {
    let mut endpoint = None;
    let mut polling_interval_seconds = None;
    let mut websocket_enabled = None;
    let mut selected_filter = None;
    let mut theme_preference = None;
    let mut confirm_destructive_actions = None;
    let mut notify_download_outcomes = None;
    let mut new_download_directory = None;
    let mut new_download_output_filename = None;
    let mut new_download_max_download_limit = None;
    let mut new_download_max_upload_limit = None;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once('=').ok_or(())?;
        let key = key.trim();
        let value = value.trim();

        match key {
            "endpoint" => endpoint = Some(value.to_owned()),
            "polling_interval_seconds" => {
                polling_interval_seconds = Some(value.parse::<u16>().map_err(|_| ())?);
            }
            "websocket_enabled" => {
                websocket_enabled = parse_bool(value);
            }
            "selected_filter" => selected_filter = Some(value.to_owned()),
            "theme" => theme_preference = ThemePreference::from_config_value(value),
            "confirm_destructive_actions" => {
                confirm_destructive_actions = parse_bool(value);
            }
            "notify_download_outcomes" => {
                notify_download_outcomes = parse_bool(value);
            }
            "new_download_directory" => new_download_directory = Some(value.to_owned()),
            "new_download_output_filename" => {
                new_download_output_filename = Some(value.to_owned());
            }
            "new_download_max_download_limit" => {
                new_download_max_download_limit = Some(value.to_owned());
            }
            "new_download_max_upload_limit" => {
                new_download_max_upload_limit = Some(value.to_owned());
            }
            "auth" if value == "session-only" || value == "none" => {}
            _ => {}
        }
    }

    let settings = Settings::new_without_secret(
        endpoint.unwrap_or_else(|| DEFAULT_ENDPOINT.to_owned()),
        polling_interval_seconds.unwrap_or(DEFAULT_POLLING_INTERVAL_SECONDS),
    )
    .map_err(|_| ())?;
    let mut settings = settings;
    settings.websocket_enabled = websocket_enabled.unwrap_or(true);

    Ok(ConfigLoad {
        config: PersistedConfig::with_auth_storage(
            settings,
            selected_filter.unwrap_or_else(|| "active".to_owned()),
            AuthStorage::None,
        )
        .with_daemon_mode(DaemonMode::External)
        .with_theme_preference(theme_preference.unwrap_or(ThemePreference::System))
        .with_ui_preferences(
            confirm_destructive_actions.unwrap_or(true),
            notify_download_outcomes.unwrap_or(false),
        )
        .with_new_download_directory(new_download_directory.unwrap_or_default())
        .with_new_download_defaults(
            new_download_output_filename.unwrap_or_default(),
            new_download_max_download_limit.unwrap_or_default(),
            new_download_max_upload_limit.unwrap_or_default(),
        ),
        feedback: None,
    })
}

fn serialize_config(config: &PersistedConfig) -> Result<String, toml::ser::Error> {
    let plaintext_token = match (config.auth_storage(), config.settings().auth()) {
        (AuthStorage::PlaintextFallback, RpcAuth::SessionSecret(secret)) => {
            Some(secret.expose_for_session().to_owned())
        }
        _ => None,
    };

    toml::to_string_pretty(&TomlConfig {
        version: Some(1),
        daemon: Some(TomlDaemon {
            mode: Some(config.daemon_mode().into()),
        }),
        connection: Some(TomlConnection {
            endpoint: Some(config.settings().endpoint().to_owned()),
            polling_interval_seconds: Some(config.settings().polling_interval_seconds()),
            websocket_enabled: Some(config.settings().websocket_enabled()),
        }),
        auth: Some(TomlAuth {
            storage: Some(config.auth_storage().into()),
            plaintext_token,
        }),
        ui: Some(TomlUi {
            selected_filter: Some(config.selected_filter().to_owned()),
            theme: Some(config.theme_preference().into()),
            confirm_destructive_actions: Some(config.confirm_destructive_actions()),
            notify_download_outcomes: Some(config.notify_download_outcomes()),
        }),
        new_download: Some(TomlNewDownload {
            directory: string_to_optional(config.new_download_directory()),
            output_filename: string_to_optional(config.new_download_output_filename()),
            max_download_limit: string_to_optional(config.new_download_max_download_limit()),
            max_upload_limit: string_to_optional(config.new_download_max_upload_limit()),
        }),
    })
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn string_to_optional(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcAuth {
    NoSecret,
    SessionSecret(Secret),
}

impl RpcAuth {
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::NoSecret => "No authentication",
            Self::SessionSecret(_) => "Token secret",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn session(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub(crate) fn expose_for_session(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SettingsDraft {
    endpoint: String,
    secret: String,
    polling_interval_seconds: u16,
    websocket_enabled: bool,
}

impl SettingsDraft {
    pub fn from_settings(settings: &Settings) -> Self {
        let secret = match settings.auth() {
            RpcAuth::NoSecret => String::new(),
            RpcAuth::SessionSecret(secret) => secret.expose_for_session().to_owned(),
        };

        Self {
            endpoint: settings.endpoint().to_owned(),
            secret,
            polling_interval_seconds: settings.polling_interval_seconds(),
            websocket_enabled: settings.websocket_enabled(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.endpoint = endpoint.into();
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn set_secret(&mut self, secret: impl Into<String>) {
        self.secret = secret.into();
    }

    pub fn polling_interval_seconds(&self) -> u16 {
        self.polling_interval_seconds
    }

    pub fn set_polling_interval_seconds(&mut self, seconds: u16) {
        self.polling_interval_seconds = seconds.max(1);
    }

    pub fn websocket_enabled(&self) -> bool {
        self.websocket_enabled
    }

    pub fn set_websocket_enabled(&mut self, enabled: bool) {
        self.websocket_enabled = enabled;
    }

    pub fn apply(&self) -> Result<Settings, SettingsDraftError> {
        Settings::validate_endpoint(&self.endpoint).map_err(SettingsDraftError::Endpoint)?;

        let auth = if self.secret.is_empty() {
            RpcAuth::NoSecret
        } else {
            RpcAuth::SessionSecret(Secret::session(&self.secret))
        };

        Ok(Settings {
            endpoint: self.endpoint.trim().to_owned(),
            auth,
            polling_interval_seconds: self.polling_interval_seconds,
            websocket_enabled: self.websocket_enabled,
        })
    }

    pub fn cancel_to(&mut self, settings: &Settings) {
        *self = Self::from_settings(settings);
    }

    pub fn endpoint_validation_message(&self) -> Option<&'static str> {
        Settings::validate_endpoint(&self.endpoint)
            .err()
            .map(EndpointValidationError::message)
    }
}

impl fmt::Debug for SettingsDraft {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SettingsDraft")
            .field("endpoint", &self.endpoint)
            .field("secret", &"<redacted>")
            .field("polling_interval_seconds", &self.polling_interval_seconds)
            .field("websocket_enabled", &self.websocket_enabled)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDraftError {
    Endpoint(EndpointValidationError),
}

impl SettingsDraftError {
    #[allow(dead_code)]
    pub fn message(self) -> &'static str {
        match self {
            Self::Endpoint(error) => error.message(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointValidationError {
    Empty,
    UnsupportedScheme,
    MissingHost,
    CredentialsNotAllowed,
    ContainsWhitespace,
}

impl EndpointValidationError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Empty => "Endpoint is required.",
            Self::UnsupportedScheme => "Endpoint must start with http:// or https://.",
            Self::MissingHost => "Endpoint must include a host.",
            Self::CredentialsNotAllowed => "Endpoint must not include credentials.",
            Self::ContainsWhitespace => "Endpoint must not contain whitespace.",
        }
    }
}

fn validate_endpoint(endpoint: &str) -> Result<(), EndpointValidationError> {
    let trimmed = endpoint.trim();

    if trimmed.is_empty() {
        return Err(EndpointValidationError::Empty);
    }

    if trimmed.chars().any(char::is_whitespace) {
        return Err(EndpointValidationError::ContainsWhitespace);
    }

    let without_scheme = if let Some(rest) = trimmed.strip_prefix("http://") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        rest
    } else {
        return Err(EndpointValidationError::UnsupportedScheme);
    };

    let authority = without_scheme
        .split_once('/')
        .map_or(without_scheme, |(authority, _path)| authority);

    if authority.is_empty() {
        return Err(EndpointValidationError::MissingHost);
    }

    if authority.contains('@') {
        return Err(EndpointValidationError::CredentialsNotAllowed);
    }

    Ok(())
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlConfig {
    version: Option<u16>,
    daemon: Option<TomlDaemon>,
    connection: Option<TomlConnection>,
    auth: Option<TomlAuth>,
    ui: Option<TomlUi>,
    new_download: Option<TomlNewDownload>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlDaemon {
    mode: Option<TomlDaemonMode>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlConnection {
    endpoint: Option<String>,
    polling_interval_seconds: Option<u16>,
    websocket_enabled: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlAuth {
    storage: Option<TomlAuthStorage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plaintext_token: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlUi {
    selected_filter: Option<String>,
    theme: Option<TomlThemePreference>,
    confirm_destructive_actions: Option<bool>,
    notify_download_outcomes: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct TomlNewDownload {
    #[serde(skip_serializing_if = "Option::is_none")]
    directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_download_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_upload_limit: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TomlDaemonMode {
    Managed,
    External,
}

impl From<TomlDaemonMode> for DaemonMode {
    fn from(value: TomlDaemonMode) -> Self {
        match value {
            TomlDaemonMode::Managed => Self::Managed,
            TomlDaemonMode::External => Self::External,
        }
    }
}

impl From<DaemonMode> for TomlDaemonMode {
    fn from(value: DaemonMode) -> Self {
        match value {
            DaemonMode::Managed => Self::Managed,
            DaemonMode::External => Self::External,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TomlThemePreference {
    System,
    Light,
    Dark,
}

impl From<TomlThemePreference> for ThemePreference {
    fn from(value: TomlThemePreference) -> Self {
        match value {
            TomlThemePreference::System => Self::System,
            TomlThemePreference::Light => Self::Light,
            TomlThemePreference::Dark => Self::Dark,
        }
    }
}

impl From<ThemePreference> for TomlThemePreference {
    fn from(value: ThemePreference) -> Self {
        match value {
            ThemePreference::System => Self::System,
            ThemePreference::Light => Self::Light,
            ThemePreference::Dark => Self::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum TomlAuthStorage {
    None,
    Keyring,
    PlaintextFallback,
    SessionOnly,
}

impl From<TomlAuthStorage> for AuthStorage {
    fn from(value: TomlAuthStorage) -> Self {
        match value {
            TomlAuthStorage::None => Self::None,
            TomlAuthStorage::Keyring => Self::Keyring,
            TomlAuthStorage::PlaintextFallback => Self::PlaintextFallback,
            TomlAuthStorage::SessionOnly => Self::SessionOnly,
        }
    }
}

impl From<AuthStorage> for TomlAuthStorage {
    fn from(value: AuthStorage) -> Self {
        match value {
            AuthStorage::None => Self::None,
            AuthStorage::Keyring => Self::Keyring,
            AuthStorage::PlaintextFallback => Self::PlaintextFallback,
            AuthStorage::SessionOnly => Self::SessionOnly,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        AuthStorage, DaemonMode, EndpointValidationError, PersistedConfig, RpcAuth, Secret,
        Settings, SettingsDraft, ThemePreference, TokenStore, TokenStoreError,
        load_config_with_token_store, save_config_with_token_store,
        save_config_without_token_store,
    };

    #[test]
    fn default_config_selects_managed_mode() {
        let config = PersistedConfig::default();

        assert_eq!(config.daemon_mode(), DaemonMode::Managed);
    }

    #[test]
    fn external_settings_default_to_local_aria2_without_secret() {
        let settings = Settings::default();

        assert_eq!(settings.endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(settings.auth(), &RpcAuth::NoSecret);
        assert_eq!(settings.polling_interval_seconds(), 2);
        assert!(settings.websocket_enabled());
    }

    #[test]
    fn validates_endpoint_drafts() {
        assert_eq!(
            Settings::validate_endpoint(""),
            Err(EndpointValidationError::Empty)
        );
        assert_eq!(
            Settings::validate_endpoint("ftp://localhost:6800/jsonrpc"),
            Err(EndpointValidationError::UnsupportedScheme)
        );
        assert_eq!(
            Settings::validate_endpoint("http://user:pass@localhost/jsonrpc"),
            Err(EndpointValidationError::CredentialsNotAllowed)
        );
        assert!(Settings::validate_endpoint("http://localhost:6800/jsonrpc").is_ok());
    }

    #[test]
    fn draft_settings_apply_and_cancel_separately_from_applied_settings() {
        let applied = Settings::default();
        let mut draft = SettingsDraft::from_settings(&applied);

        draft.set_endpoint("http://aria2.local:6800/jsonrpc");
        draft.set_secret("super-secret");
        draft.set_polling_interval_seconds(5);
        draft.set_websocket_enabled(false);

        assert_eq!(applied.endpoint(), "http://localhost:6800/jsonrpc");

        let applied_from_draft = draft.apply().expect("draft should validate");
        assert_eq!(
            applied_from_draft.endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
        assert_eq!(
            applied_from_draft.auth(),
            &RpcAuth::SessionSecret(Secret::session("super-secret"))
        );
        assert_eq!(applied_from_draft.polling_interval_seconds(), 5);
        assert!(!applied_from_draft.websocket_enabled());

        draft.cancel_to(&applied);
        assert_eq!(draft.endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(draft.secret(), "");
        assert_eq!(draft.polling_interval_seconds(), 2);
        assert!(draft.websocket_enabled());
    }

    #[test]
    fn secrets_are_redacted_from_debug_and_display_safe_text() {
        let secret = Secret::session("super-secret");
        let auth = RpcAuth::SessionSecret(secret);
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_secret("super-secret");

        assert!(!format!("{auth:?}").contains("super-secret"));
        assert!(!format!("{draft:?}").contains("super-secret"));
        assert_eq!(auth.display_label(), "Token secret");
    }

    #[test]
    fn empty_secret_applies_without_authentication() {
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_secret("");

        let settings = draft.apply().expect("empty secret disables auth");

        assert_eq!(settings.auth(), &RpcAuth::NoSecret);
    }

    #[test]
    fn saves_and_loads_toml_config_without_secret() {
        let path = temp_config_path("save-load");
        let token_store = MemoryTokenStore::default();
        let settings =
            Settings::new_without_secret("http://aria2.local:6800/jsonrpc", 5).expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "paused", AuthStorage::None);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config file");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(contents.contains("[daemon]"));
        assert!(contents.contains("mode = \"external\""));
        assert!(contents.contains("websocket_enabled = true"));
        assert_eq!(loaded.feedback(), None);
        assert_eq!(loaded.config().daemon_mode(), DaemonMode::External);
        assert_eq!(
            loaded.config().settings().endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
        assert_eq!(loaded.config().settings().polling_interval_seconds(), 5);
        assert!(loaded.config().settings().websocket_enabled());
        assert_eq!(loaded.config().settings().auth(), &RpcAuth::NoSecret);
        assert_eq!(loaded.config().selected_filter(), "paused");
        assert_eq!(loaded.config().theme_preference(), ThemePreference::System);
    }

    #[test]
    fn saves_and_loads_theme_preference() {
        let path = temp_config_path("theme-preference");
        let token_store = MemoryTokenStore::default();
        let settings = Settings::default();
        let config = PersistedConfig::with_auth_storage_and_theme(
            settings,
            "all",
            AuthStorage::None,
            ThemePreference::Dark,
        );

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config written");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(contents.contains("theme = \"dark\""));
        assert_eq!(loaded.config().theme_preference(), ThemePreference::Dark);
    }

    #[test]
    fn tokenless_save_writes_ui_preferences_without_touching_keyring() {
        let path = temp_config_path("theme-no-token-store");
        let settings = Settings::new(
            "http://aria2.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("super-secret")),
            3,
            true,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage_and_theme(
            settings,
            "active",
            AuthStorage::Keyring,
            ThemePreference::Light,
        );

        save_config_without_token_store(&path, &config).expect("config saves without keyring");
        let contents = fs::read_to_string(&path).expect("config file");

        assert!(contents.contains("theme = \"light\""));
        assert!(!contents.contains("super-secret"));
    }

    #[test]
    fn legacy_key_value_config_still_loads() {
        let path = temp_config_path("legacy");
        fs::write(
            &path,
            "endpoint=http://aria2.local:6800/jsonrpc\npolling_interval_seconds=7\nselected_filter=error\nauth=session-only\n",
        )
        .expect("legacy config");

        let loaded = load_config_with_token_store(&path, &MemoryTokenStore::default());

        assert_eq!(loaded.config().daemon_mode(), DaemonMode::External);
        assert_eq!(
            loaded.config().settings().endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
        assert_eq!(loaded.config().settings().polling_interval_seconds(), 7);
        assert!(loaded.config().settings().websocket_enabled());
        assert_eq!(loaded.config().selected_filter(), "error");
        assert_eq!(loaded.config().theme_preference(), ThemePreference::System);
    }

    #[test]
    fn pre_mode_toml_config_loads_as_external_mode() {
        let path = temp_config_path("pre-mode-toml");
        fs::write(
            &path,
            r#"
version = 1

[connection]
endpoint = "http://aria2.local:6800/jsonrpc"
polling_interval_seconds = 7
websocket_enabled = false

[auth]
storage = "none"
"#,
        )
        .expect("pre-mode config");

        let loaded = load_config_with_token_store(&path, &MemoryTokenStore::default());

        assert_eq!(loaded.config().daemon_mode(), DaemonMode::External);
        assert_eq!(
            loaded.config().settings().endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
    }

    #[test]
    fn managed_mode_persists_external_settings_without_managed_endpoint_or_secret() {
        let path = temp_config_path("managed-preserves-external");
        let token_store = MemoryTokenStore::default();
        let settings = Settings::new(
            "http://external.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("external-secret")),
            3,
            false,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "active", AuthStorage::Keyring)
            .with_daemon_mode(DaemonMode::Managed);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config file");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(contents.contains("mode = \"managed\""));
        assert!(contents.contains("endpoint = \"http://external.local:6800/jsonrpc\""));
        assert!(!contents.contains("external-secret"));
        assert_eq!(loaded.config().daemon_mode(), DaemonMode::Managed);
        assert_eq!(
            loaded.config().settings().auth(),
            &RpcAuth::SessionSecret(Secret::session("external-secret"))
        );
    }

    #[test]
    fn invalid_config_recovers_to_defaults_with_feedback() {
        let path = temp_config_path("invalid");
        fs::write(&path, "endpoint=ftp://bad\n").expect("write invalid config");

        let loaded = load_config_with_token_store(&path, &MemoryTokenStore::default());

        assert_eq!(loaded.config().daemon_mode(), DaemonMode::Managed);
        assert_eq!(loaded.config().settings(), &Settings::default());
        assert_eq!(
            loaded.feedback(),
            Some("Config was invalid; using defaults.")
        );
    }

    #[test]
    fn keyring_secret_is_stored_outside_config_file_and_restored() {
        let path = temp_config_path("keyring-secret");
        let token_store = MemoryTokenStore::default();
        let settings = Settings::new(
            "http://aria2.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("super-secret")),
            3,
            true,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "all", AuthStorage::Keyring);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config file");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(!contents.contains("super-secret"));
        assert_eq!(
            loaded.config().settings().auth(),
            &RpcAuth::SessionSecret(Secret::session("super-secret"))
        );
    }

    #[test]
    fn plaintext_fallback_secret_is_persisted_only_when_requested() {
        let path = temp_config_path("plaintext-secret");
        let token_store = MemoryTokenStore::default();
        let settings = Settings::new(
            "http://aria2.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("super-secret")),
            3,
            true,
        )
        .expect("settings");
        let config =
            PersistedConfig::with_auth_storage(settings, "all", AuthStorage::PlaintextFallback);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config file");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(contents.contains("super-secret"));
        assert_eq!(
            loaded.config().settings().auth(),
            &RpcAuth::SessionSecret(Secret::session("super-secret"))
        );
    }

    #[test]
    fn session_only_secret_is_not_persisted() {
        let path = temp_config_path("session-only");
        let token_store = MemoryTokenStore::default();
        let settings = Settings::new(
            "http://aria2.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("super-secret")),
            3,
            true,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "all", AuthStorage::SessionOnly);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");
        let contents = fs::read_to_string(&path).expect("config file");
        let loaded = load_config_with_token_store(&path, &token_store);

        assert!(!contents.contains("super-secret"));
        assert_eq!(loaded.config().settings().auth(), &RpcAuth::NoSecret);
    }

    #[test]
    fn keyring_store_failure_blocks_secure_secret_save() {
        let path = temp_config_path("keyring-fails");
        let token_store = MemoryTokenStore::failing();
        let settings = Settings::new(
            "http://aria2.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("super-secret")),
            3,
            true,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "all", AuthStorage::Keyring);

        let error = save_config_with_token_store(&path, &config, None, &token_store)
            .expect_err("secure storage should fail");

        assert!(error.is_token_store_error());
    }

    #[test]
    fn clearing_secret_deletes_keyring_token_for_endpoint() {
        let path = temp_config_path("clear-secret");
        let token_store = MemoryTokenStore::default();
        token_store
            .save(
                "http://aria2.local:6800/jsonrpc",
                &Secret::session("super-secret"),
            )
            .expect("stored token");
        let settings =
            Settings::new_without_secret("http://aria2.local:6800/jsonrpc", 3).expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "all", AuthStorage::None);

        save_config_with_token_store(&path, &config, None, &token_store).expect("config saves");

        assert_eq!(
            token_store
                .load("http://aria2.local:6800/jsonrpc")
                .expect("load token"),
            None
        );
    }

    #[test]
    fn endpoint_change_deletes_old_keyring_token() {
        let path = temp_config_path("delete-old");
        let token_store = MemoryTokenStore::default();
        token_store
            .save("http://old.local:6800/jsonrpc", &Secret::session("old"))
            .expect("old token");
        let settings = Settings::new(
            "http://new.local:6800/jsonrpc",
            RpcAuth::SessionSecret(Secret::session("new")),
            3,
            true,
        )
        .expect("settings");
        let config = PersistedConfig::with_auth_storage(settings, "all", AuthStorage::Keyring);

        save_config_with_token_store(
            &path,
            &config,
            Some("http://old.local:6800/jsonrpc"),
            &token_store,
        )
        .expect("config saves");

        assert_eq!(
            token_store
                .load("http://old.local:6800/jsonrpc")
                .expect("load old"),
            None
        );
    }

    fn temp_config_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cottid-{name}-{unique}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir.join("config")
    }

    #[derive(Default)]
    struct MemoryTokenStore {
        tokens: RefCell<BTreeMap<String, Secret>>,
        fail: bool,
    }

    impl MemoryTokenStore {
        fn failing() -> Self {
            Self {
                tokens: RefCell::new(BTreeMap::new()),
                fail: true,
            }
        }
    }

    impl TokenStore for MemoryTokenStore {
        fn load(&self, endpoint: &str) -> Result<Option<Secret>, TokenStoreError> {
            if self.fail {
                return Err(TokenStoreError::new("token store failed"));
            }

            Ok(self.tokens.borrow().get(endpoint).cloned())
        }

        fn save(&self, endpoint: &str, secret: &Secret) -> Result<(), TokenStoreError> {
            if self.fail {
                return Err(TokenStoreError::new("token store failed"));
            }

            self.tokens
                .borrow_mut()
                .insert(endpoint.to_owned(), secret.clone());
            Ok(())
        }

        fn delete(&self, endpoint: &str) -> Result<(), TokenStoreError> {
            if self.fail {
                return Err(TokenStoreError::new("token store failed"));
            }

            self.tokens.borrow_mut().remove(endpoint);
            Ok(())
        }
    }
}
