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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub id: Uuid,
    pub username: String,
    pub disabled: bool,
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

    pub fn list(&self) -> Vec<AccountSummary> {
        let accounts = self
            .accounts
            .read()
            .unwrap_or_else(|error| error.into_inner());
        let mut summaries: Vec<_> = accounts
            .accounts
            .iter()
            .map(AccountRecord::summary)
            .collect();
        summaries.sort_by(|left, right| left.username.cmp(&right.username));
        summaries
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
        let mut updated = accounts.clone();
        updated.accounts.push(AccountRecord {
            id,
            username: username.to_string(),
            password_hash: hash_password(password)?,
            disabled: false,
        });
        self.persist(&updated)?;
        *accounts = updated;
        Ok(id)
    }

    pub fn create(&self, username: &str, password: &str) -> Result<AccountSummary, String> {
        validate_username(username)?;
        validate_password(password)?;
        let mut accounts = self
            .accounts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        if accounts
            .accounts
            .iter()
            .any(|account| account.username == username)
        {
            return Err("An application account with this username already exists".to_string());
        }
        let account = AccountRecord {
            id: Uuid::new_v4(),
            username: username.to_string(),
            password_hash: hash_password(password)?,
            disabled: false,
        };
        let summary = account.summary();
        let mut updated = accounts.clone();
        updated.accounts.push(account);
        self.persist(&updated)?;
        *accounts = updated;
        Ok(summary)
    }

    pub fn set_disabled(&self, id: Uuid, disabled: bool) -> Result<AccountSummary, String> {
        let mut accounts = self
            .accounts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let mut updated = accounts.clone();
        let account_index = updated
            .accounts
            .iter()
            .position(|account| account.id == id)
            .ok_or_else(|| "Application account not found".to_string())?;
        updated.accounts[account_index].disabled = disabled;
        if disabled && !updated.accounts.iter().any(|account| !account.disabled) {
            return Err("The last active application account cannot be disabled".to_string());
        }
        let summary = updated.accounts[account_index].summary();
        self.persist(&updated)?;
        *accounts = updated;
        Ok(summary)
    }

    pub fn change_password(&self, id: Uuid, password: &str) -> Result<(), String> {
        validate_password(password)?;
        let password_hash = hash_password(password)?;
        let mut accounts = self
            .accounts
            .write()
            .unwrap_or_else(|error| error.into_inner());
        let mut updated = accounts.clone();
        let account = updated
            .accounts
            .iter_mut()
            .find(|account| account.id == id)
            .ok_or_else(|| "Application account not found".to_string())?;
        account.password_hash = password_hash;
        self.persist(&updated)?;
        *accounts = updated;
        Ok(())
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

impl AccountRecord {
    fn summary(&self) -> AccountSummary {
        AccountSummary {
            id: self.id,
            username: self.username.clone(),
            disabled: self.disabled,
        }
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
