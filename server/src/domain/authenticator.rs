use async_trait::async_trait;
use common::domain::EnvVar;
use common::domain::Error;
use common::domain::error::{AuthenticationFailedError, UserAlreadyExistsError, UserNotFoundError};
use common::dto::{
    LoginRequestDto, PasskeyFinishLoginRequestDto, PasskeyFinishRegistrationRequestDto,
    PasskeyOptionsDto, PasskeyStartLoginRequestDto, PasskeyStartRegistrationRequestDto,
    RecoverUserRequestDto, RecoveryCodeDto, RegistrationRequestDto, UpdateUserProfileRequestDto, UserDto, UserSettingsDto,
};
use crate::ports::{AuthDrivingPort, UserRepositoryDrivenPort};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use webauthn_rs::prelude::{
    PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential, RegisterPublicKeyCredential,
    Url, Webauthn, WebauthnBuilder,
};

const RECOVERY_CODE_TTL_SECONDS: i64 = 15 * 60;

struct PendingPasskeyRegistration {
    user_handle: Uuid,
    state: PasskeyRegistration,
}

pub trait AuthenticatorDrivenPorts: Send + Sync + 'static {
    type UserRepo: UserRepositoryDrivenPort;
}

pub struct Authenticator<DP: AuthenticatorDrivenPorts> {
    user_repo: DP::UserRepo,
    webauthn: Webauthn,
    pending_passkey_registrations: Mutex<HashMap<String, PendingPasskeyRegistration>>,
    pending_passkey_authentications: Mutex<HashMap<String, PasskeyAuthentication>>,
}

impl<DP: AuthenticatorDrivenPorts> Authenticator<DP> {
    pub fn new(user_repo: DP::UserRepo) -> Self {
        Self::with_webauthn(
            user_repo,
            build_webauthn().expect("Failed to construct WebAuthn service"),
        )
    }

    pub fn with_webauthn(user_repo: DP::UserRepo, webauthn: Webauthn) -> Self {
        Self {
            user_repo,
            webauthn,
            pending_passkey_registrations: Mutex::new(HashMap::new()),
            pending_passkey_authentications: Mutex::new(HashMap::new()),
        }
    }

    fn default_user_settings() -> UserSettingsDto {
        UserSettingsDto {
            turn_left_key: "A".to_string(),
            turn_right_key: "D".to_string(),
            thrust_key: "W".to_string(),
            music_enabled: true,
            music_volume: 45,
            sound_effects_enabled: true,
            sound_effects_volume: 60,
        }
    }

    fn current_timestamp() -> Result<i64, Error> {
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?
            .as_secs() as i64)
    }
}

#[async_trait]
impl<DP: AuthenticatorDrivenPorts> AuthDrivingPort for Authenticator<DP> {
    async fn login_user(&self, login_request: LoginRequestDto) -> Result<UserDto, Error> {
        if self.user_repo.find_by_name(&login_request.name).await?.is_some() {
            return Err(Error::UserAlreadyExists(UserAlreadyExistsError::new(login_request.name)));
        }

        self.user_repo.save_user(&login_request.name, Uuid::new_v4()).await
    }

    async fn register_user(&self, registration_request: RegistrationRequestDto) -> Result<UserDto, Error> {
        if self.user_repo.find_by_name(&registration_request.name).await?.is_some() {
            return Err(Error::UserAlreadyExists(
                common::domain::error::UserAlreadyExistsError::new(registration_request.name),
            ));
        }
        self.user_repo.save_user(&registration_request.name, Uuid::new_v4()).await
    }

    async fn start_passkey_registration(&self, request: PasskeyStartRegistrationRequestDto) -> Result<PasskeyOptionsDto, Error> {
        if self.user_repo.find_by_name(&request.name).await?.is_some() {
            return Err(Error::UserAlreadyExists(
                common::domain::error::UserAlreadyExistsError::new(request.name),
            ));
        }

        let user_handle = Uuid::new_v4();
        let (options, state) = self
            .webauthn
            .start_passkey_registration(user_handle, &request.name, &request.name, None)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        self.pending_passkey_registrations
            .lock()
            .expect("passkey registration state poisoned")
            .insert(
                request.name,
                PendingPasskeyRegistration {
                    user_handle,
                    state,
                },
            );

        let mut public_key = extract_public_key_json(options)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;
        simplify_registration_options(&mut public_key);

        Ok(PasskeyOptionsDto { public_key })
    }

    async fn finish_passkey_registration(&self, request: PasskeyFinishRegistrationRequestDto) -> Result<UserDto, Error> {
        let pending_registration = self.pending_passkey_registrations
            .lock()
            .expect("passkey registration state poisoned")
            .remove(&request.name)
            .ok_or_else(|| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let credential: RegisterPublicKeyCredential = serde_json::from_value(request.credential)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let passkey = self
            .webauthn
            .finish_passkey_registration(&credential, &pending_registration.state)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let user = self.user_repo.save_user(&request.name, pending_registration.user_handle).await?;
        self.user_repo.save_passkey(&request.name, &passkey).await?;
        Ok(user)
    }

    async fn start_passkey_login(&self, request: PasskeyStartLoginRequestDto) -> Result<PasskeyOptionsDto, Error> {
        let passkeys = self.user_repo.list_passkeys_by_name(&request.name).await?;
        if passkeys.is_empty() {
            return Err(Error::UserNotFound(UserNotFoundError::new(request.name)));
        }

        let (options, state) = self
            .webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        self.pending_passkey_authentications
            .lock()
            .expect("passkey authentication state poisoned")
            .insert(request.name, state);

        Ok(PasskeyOptionsDto {
            public_key: extract_public_key_json(options)
                .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?,
        })
    }

    async fn finish_passkey_login(&self, request: PasskeyFinishLoginRequestDto) -> Result<UserDto, Error> {
        let state = self.pending_passkey_authentications
            .lock()
            .expect("passkey authentication state poisoned")
            .remove(&request.name)
            .ok_or_else(|| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let credential: PublicKeyCredential = serde_json::from_value(request.credential)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let authentication_result = self
            .webauthn
            .finish_passkey_authentication(&credential, &state)
            .map_err(|_| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let mut passkeys = self.user_repo.list_passkeys_by_name(&request.name).await?;
        let passkey = passkeys
            .iter_mut()
            .find(|entry| entry.cred_id() == authentication_result.cred_id())
            .ok_or_else(|| Error::AuthenticationFailed(AuthenticationFailedError::new()))?;

        let _ = passkey.update_credential(&authentication_result);
        self.user_repo.update_passkey(&request.name, passkey).await?;

        self.user_repo
            .find_by_name(&request.name)
            .await?
            .ok_or_else(|| Error::UserNotFound(UserNotFoundError::new(request.name)))
    }

    async fn create_recovery_code(&self, user_name: String) -> Result<RecoveryCodeDto, Error> {
        let recovery_code = Uuid::new_v4().simple().to_string().to_uppercase();
        let expires_at = Self::current_timestamp()? + RECOVERY_CODE_TTL_SECONDS;
        self.user_repo
            .create_recovery_code(&user_name, &recovery_code, expires_at)
            .await?;

        Ok(RecoveryCodeDto {
            recovery_code,
            expires_at,
        })
    }

    async fn recover_user(&self, request: RecoverUserRequestDto) -> Result<UserDto, Error> {
        let now = Self::current_timestamp()?;
        let user = self.user_repo
            .find_by_recovery_code(&request.recovery_code, now)
            .await?
            .ok_or_else(|| Error::UserNotFound(UserNotFoundError::new(request.recovery_code.clone())))?;
        self.user_repo.mark_recovery_code_used(&request.recovery_code).await?;
        Ok(user)
    }

    async fn update_user_profile(&self, current_user_name: String, request: UpdateUserProfileRequestDto) -> Result<UserDto, Error> {
        if request.name != current_user_name
            && self.user_repo.find_by_name(&request.name).await?.is_some()
        {
            return Err(Error::UserAlreadyExists(
                common::domain::error::UserAlreadyExistsError::new(request.name),
            ));
        }

        self.user_repo
            .update_user_profile(&current_user_name, &request.name, &request.profile_image_url)
            .await
    }

    async fn get_user_settings(&self, user_name: String) -> Result<UserSettingsDto, Error> {
        Ok(self.user_repo
            .find_settings_by_name(&user_name)
            .await?
            .unwrap_or_else(Self::default_user_settings))
    }

    async fn save_user_settings(&self, user_name: String, settings: UserSettingsDto) -> Result<UserSettingsDto, Error> {
        self.user_repo.save_settings(&user_name, &settings).await
    }
}

fn build_webauthn() -> Result<Webauthn, String> {
    let rp_id = EnvVar::ServerWebauthnRpId.value();
    let server_origin = Url::parse(&EnvVar::ServerWebauthnOrigin.value()).map_err(|error| error.to_string())?;
    let mut builder = WebauthnBuilder::new(&rp_id, &server_origin)
        .map_err(|error| error.to_string())?;
    builder = builder.rp_name("Battle Control");

    for allowed_origin in webauthn_allowed_origins() {
        let allowed_origin = Url::parse(&allowed_origin).map_err(|error| error.to_string())?;
        builder = builder.append_allowed_origin(&allowed_origin);
    }

    builder.build().map_err(|error| error.to_string())
}

fn webauthn_allowed_origins() -> Vec<String> {
    EnvVar::ServerWebauthnAllowedOrigins
        .value()
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_string)
        .collect()
}

fn simplify_registration_options(options: &mut serde_json::Value) {
    let target = if options.get("publicKey").is_some() {
        &mut options["publicKey"]
    } else {
        options
    };
    if let Some(auth_selection) = target.get_mut("authenticatorSelection") {
        auth_selection["residentKey"] = "preferred".into();
        auth_selection["userVerification"] = "preferred".into();
    }
    if let Some(extensions) = target.get_mut("extensions") {
        *extensions = serde_json::json!({ "credProps": true });
    }
}

fn extract_public_key_json<T: ::serde::Serialize>(options: T) -> Result<serde_json::Value, serde_json::Error> {
    let value = serde_json::to_value(options)?;
    Ok(value
        .get("publicKey")
        .cloned()
        .unwrap_or(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::AuthDrivingPort;
    use crate::test_helpers::FakeUserRepository;
    use crate::test_helpers::sample_data::{test_registration_request, TEST_PLAYER_NAME};

    struct FakeDrivenPorts;
    impl AuthenticatorDrivenPorts for FakeDrivenPorts {
        type UserRepo = FakeUserRepository;
    }

    #[test]
    fn webauthn_allowed_origins_returns_default_origins() {
        unsafe {
            std::env::remove_var("MATTER_SERVER_WEBAUTHN_ALLOWED_ORIGINS");
        }

        assert_eq!(webauthn_allowed_origins(), vec![
            "http://localhost:5173".to_string(),
            "http://localhost:5175".to_string(),
        ]);
    }

    #[test]
    fn webauthn_allowed_origins_splits_env_var() {
        unsafe {
            std::env::set_var(
                "MATTER_SERVER_WEBAUTHN_ALLOWED_ORIGINS",
                "https://battlecontrol.io,https://www.battlecontrol.io",
            );
        }

        assert_eq!(webauthn_allowed_origins(), vec![
            "https://battlecontrol.io".to_string(),
            "https://www.battlecontrol.io".to_string(),
        ]);

        unsafe {
            std::env::remove_var("MATTER_SERVER_WEBAUTHN_ALLOWED_ORIGINS");
        }
    }

    #[tokio::test]
    async fn login_user_returns_error_when_user_exists() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth
            .login_user(LoginRequestDto { name: TEST_PLAYER_NAME.to_string() })
            .await;
        assert!(matches!(result, Err(Error::UserAlreadyExists(_))));
    }

    #[tokio::test]
    async fn login_user_saves_user_when_name_is_available() {
        let repo = FakeUserRepository::new();
        let repo_clone = repo.clone();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let _ = auth
            .login_user(LoginRequestDto { name: TEST_PLAYER_NAME.to_string() })
            .await
            .unwrap();
        assert_eq!(repo_clone.save_user_calls().len(), 1);
    }

    #[tokio::test]
    async fn register_user_does_not_save_when_user_exists() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let repo_clone = repo.clone();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let _ = auth.register_user(test_registration_request()).await;
        assert_eq!(repo_clone.save_user_calls().len(), 0);
    }

    #[tokio::test]
    async fn register_user_returns_error_when_user_exists() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth.register_user(test_registration_request()).await;
        assert!(matches!(result, Err(Error::UserAlreadyExists(_))));
    }

    #[tokio::test]
    async fn register_user_saves_user_when_not_exists() {
        let repo = FakeUserRepository::new();
        let repo_clone = repo.clone();
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        auth.register_user(test_registration_request()).await.unwrap();
        let calls = repo_clone.save_user_calls();
        assert_eq!(calls.len(), 1);
    }

    #[tokio::test]
    async fn create_recovery_code_returns_code() {
        let repo = FakeUserRepository::new()
            .with_existing_user(crate::test_helpers::sample_data::test_user());
        let auth = Authenticator::<FakeDrivenPorts>::new(repo);
        let result = auth.create_recovery_code(TEST_PLAYER_NAME.to_string()).await.unwrap();
        assert!(!result.recovery_code.is_empty());
    }
}
