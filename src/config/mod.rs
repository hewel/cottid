use std::fmt;

const DEFAULT_ENDPOINT: &str = "http://localhost:6800/jsonrpc";
const DEFAULT_POLLING_INTERVAL_SECONDS: u16 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    endpoint: String,
    auth: RpcAuth,
    polling_interval_seconds: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            auth: RpcAuth::NoSecret,
            polling_interval_seconds: DEFAULT_POLLING_INTERVAL_SECONDS,
        }
    }
}

impl Settings {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn auth(&self) -> &RpcAuth {
        &self.auth
    }

    pub fn polling_interval_seconds(&self) -> u16 {
        self.polling_interval_seconds
    }

    pub fn validate_endpoint(endpoint: &str) -> Result<(), EndpointValidationError> {
        validate_endpoint(endpoint)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcAuthDraft {
    NoSecret,
    SessionSecret,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SettingsDraft {
    endpoint: String,
    auth: RpcAuthDraft,
    secret: String,
    polling_interval_seconds: u16,
}

impl SettingsDraft {
    pub fn from_settings(settings: &Settings) -> Self {
        let (auth, secret) = match settings.auth() {
            RpcAuth::NoSecret => (RpcAuthDraft::NoSecret, String::new()),
            RpcAuth::SessionSecret(secret) => (
                RpcAuthDraft::SessionSecret,
                secret.expose_for_session().to_owned(),
            ),
        };

        Self {
            endpoint: settings.endpoint().to_owned(),
            auth,
            secret,
            polling_interval_seconds: settings.polling_interval_seconds(),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.endpoint = endpoint.into();
    }

    pub fn auth(&self) -> RpcAuthDraft {
        self.auth
    }

    pub fn set_auth(&mut self, auth: RpcAuthDraft) {
        self.auth = auth;
        if matches!(auth, RpcAuthDraft::NoSecret) {
            self.secret.clear();
        }
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

    pub fn apply(&self) -> Result<Settings, SettingsDraftError> {
        Settings::validate_endpoint(&self.endpoint).map_err(SettingsDraftError::Endpoint)?;

        let auth = match self.auth {
            RpcAuthDraft::NoSecret => RpcAuth::NoSecret,
            RpcAuthDraft::SessionSecret if self.secret.is_empty() => {
                return Err(SettingsDraftError::SecretRequired);
            }
            RpcAuthDraft::SessionSecret => RpcAuth::SessionSecret(Secret::session(&self.secret)),
        };

        Ok(Settings {
            endpoint: self.endpoint.trim().to_owned(),
            auth,
            polling_interval_seconds: self.polling_interval_seconds,
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
            .field("auth", &self.auth)
            .field("secret", &"<redacted>")
            .field("polling_interval_seconds", &self.polling_interval_seconds)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsDraftError {
    Endpoint(EndpointValidationError),
    SecretRequired,
}

impl SettingsDraftError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Endpoint(error) => error.message(),
            Self::SecretRequired => "Secret is required for token authentication.",
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

#[cfg(test)]
mod tests {
    use super::{
        EndpointValidationError, RpcAuth, RpcAuthDraft, Secret, Settings, SettingsDraft,
        SettingsDraftError,
    };

    #[test]
    fn defaults_target_local_aria2_without_secret() {
        let settings = Settings::default();

        assert_eq!(settings.endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(settings.auth(), &RpcAuth::NoSecret);
        assert_eq!(settings.polling_interval_seconds(), 2);
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
        draft.set_auth(RpcAuthDraft::SessionSecret);
        draft.set_secret("super-secret");
        draft.set_polling_interval_seconds(5);

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

        draft.cancel_to(&applied);
        assert_eq!(draft.endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(draft.auth(), RpcAuthDraft::NoSecret);
        assert_eq!(draft.secret(), "");
        assert_eq!(draft.polling_interval_seconds(), 2);
    }

    #[test]
    fn secrets_are_redacted_from_debug_and_display_safe_text() {
        let secret = Secret::session("super-secret");
        let auth = RpcAuth::SessionSecret(secret);
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_auth(RpcAuthDraft::SessionSecret);
        draft.set_secret("super-secret");

        assert!(!format!("{auth:?}").contains("super-secret"));
        assert!(!format!("{draft:?}").contains("super-secret"));
        assert_eq!(auth.display_label(), "Token secret");
    }

    #[test]
    fn token_auth_requires_a_session_secret() {
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_auth(RpcAuthDraft::SessionSecret);

        let error = draft.apply().expect_err("token auth needs a secret");

        assert_eq!(error, SettingsDraftError::SecretRequired);
        assert_eq!(
            error.message(),
            "Secret is required for token authentication."
        );
    }
}
