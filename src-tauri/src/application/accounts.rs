use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

const ACCOUNTS_FILE: &str = "accounts.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountPrincipal {
    pub id: Uuid,
    pub subject: String,
}

#[derive(Clone)]
pub struct AccountStore {
    path: PathBuf,
    accounts: Arc<RwLock<AccountsFile>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct AccountsFile {
    version: u32,
    accounts: Vec<AccountRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountRecord {
    id: Uuid,
    username: String,
    password_hash: String,
    disabled: bool,
}

impl AccountStore {
    pub fn open(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join(ACCOUNTS_FILE);
        let accounts = if path.exists() {
            let bytes = std::fs::read(&path)
                .map_err(|error| format!("Failed to read application accounts: {error}"))?;
            let parsed: AccountsFile = serde_json::from_slice(&bytes)
                .map_err(|error| format!("Failed to parse application accounts: {error}"))?;
            if parsed.version != 1 {
                return Err(format!(
                    "Unsupported application accounts version {}",
                    parsed.version
                ));
            }
            parsed
        } else {
            AccountsFile {
                version: 1,
                accounts: Vec::new(),
            }
        };
        Ok(Self {
            path,
            accounts: Arc::new(RwLock::new(accounts)),
        })
    }

    pub fn has_accounts(&self) -> bool {
        !self
            .accounts
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .accounts
            .is_empty()
    }

    pub fn bootstrap_administrator(&self, username: &str, password: &str) -> Result<Uuid, String> {
        validate_username(username)?;
        validate_password(password)?;
        let mut accounts = self
            .accounts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if !accounts.accounts.is_empty() {
            return Err("Application accounts have already been initialized".to_string());
        }
        let id = Uuid::new_v4();
        accounts.accounts.push(AccountRecord {
            id,
            username: username.to_string(),
            password_hash: hash_password(password)?,
            disabled: false,
        });
        self.persist(&accounts)?;
        Ok(id)
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Option<AccountPrincipal> {
        let accounts = self
            .accounts
            .read()
            .unwrap_or_else(|error| error.into_inner());
        let account = accounts
            .accounts
            .iter()
            .find(|account| account.username == username && !account.disabled)?;
        let parsed = PasswordHash::new(&account.password_hash).ok()?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .ok()?;
        Some(AccountPrincipal {
            id: account.id,
            subject: account.username.clone(),
        })
    }

    fn persist(&self, accounts: &AccountsFile) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "Application accounts path has no parent directory".to_string())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create application account directory: {error}"))?;
        let bytes = serde_json::to_vec_pretty(accounts)
            .map_err(|error| format!("Failed to serialize application accounts: {error}"))?;
        let temporary = self.path.with_extension("json.tmp");
        crate::persistence::write_private_file(&temporary, bytes)?;
        std::fs::rename(&temporary, &self.path)
            .map_err(|error| format!("Failed to replace application accounts: {error}"))
    }
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| format!("Failed to hash application password: {error}"))
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty()
        || username.len() > 128
        || username.chars().any(char::is_control)
        || username.trim() != username
    {
        return Err(
            "Username must contain 1-128 printable characters without outer whitespace".to_string(),
        );
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 12 {
        return Err(
            "Application account passwords must contain at least 12 characters".to_string(),
        );
    }
    Ok(())
}
