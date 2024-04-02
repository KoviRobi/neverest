//! # Account configuration
//!
//! Module dedicated to account configuration.

use std::sync::Arc;

use anyhow::Result;
#[cfg(feature = "imap")]
use email::imap::config::ImapConfig;
#[cfg(feature = "maildir")]
use email::maildir::config::MaildirConfig;
#[cfg(feature = "notmuch")]
use email::notmuch::config::NotmuchConfig;
use email::{
    envelope::sync::config::EnvelopeSyncFilters,
    flag::sync::config::FlagSyncPermissions,
    folder::sync::config::{FolderSyncPermissions, FolderSyncStrategy},
    message::sync::config::MessageSyncPermissions,
};
use serde::{Deserialize, Serialize};

/// The account configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// The defaultness of the current account.
    ///
    /// When synchronizing, if no account name is explicitly given,
    /// this one will be used by default.
    pub default: Option<bool>,

    /// The account configuration dedicated to folders.
    pub folder: Option<FolderConfig>,

    /// The account configuration dedicated to envelopes.
    pub envelope: Option<EnvelopeConfig>,

    /// The configuration of the left backend.
    ///
    /// The left backend can be seen as the source backend, except
    /// that there is not implicit difference between source and
    /// target. Hence left and right are used instead.
    pub left: BackendGlobalConfig,

    /// The configuration of the right backend.
    ///
    /// The right backend can be seen as the target backend, except
    /// that there is not implicit difference between source and
    /// target. Hence left and right are used instead.
    pub right: BackendGlobalConfig,
}

impl AccountConfig {
    /// Configure the current account configuration.
    ///
    /// This function is mostly used to replace undefined keyring
    /// entries by default ones, based on the given account name.
    pub fn configure(&mut self, account_name: &str) -> Result<()> {
        match &mut self.left.backend {
            #[cfg(feature = "imap")]
            BackendConfig::Imap(config) => {
                config
                    .auth
                    .replace_undefined_keyring_entries(&account_name)?;
            }
            _ => (),
        }

        match &mut self.right.backend {
            #[cfg(feature = "imap")]
            BackendConfig::Imap(config) => {
                config
                    .auth
                    .replace_undefined_keyring_entries(&account_name)?;
            }
            _ => (),
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct FolderConfig {
    #[serde(default)]
    pub filter: FolderSyncStrategy,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EnvelopeConfig {
    #[serde(default)]
    pub filter: EnvelopeSyncFilters,
}

/// The global backend configuration (left or right).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BackendGlobalConfig {
    /// The backend-specific configuration.
    pub backend: BackendConfig,

    /// The backend configuration dedicated to folders.
    pub folder: Option<FolderBackendConfig>,

    /// The backend configuration dedicated to flags.
    pub flag: Option<FlagBackendConfig>,

    /// The backend configuration dedicated to messages.
    pub message: Option<MessageBackendConfig>,
}

impl BackendGlobalConfig {
    pub fn into_account_config(
        self,
        name: String,
        folder_filter: FolderSyncStrategy,
        envelope_filter: EnvelopeSyncFilters,
    ) -> (BackendConfig, Arc<email::account::config::AccountConfig>) {
        (
            self.backend,
            Arc::new(email::account::config::AccountConfig {
                name,
                folder: Some(email::folder::config::FolderConfig {
                    sync: Some(email::folder::sync::config::FolderSyncConfig {
                        filter: folder_filter,
                        permissions: self.folder.map(|c| c.permissions).unwrap_or_default(),
                    }),
                    ..Default::default()
                }),
                envelope: Some(email::envelope::config::EnvelopeConfig {
                    sync: Some(email::envelope::sync::config::EnvelopeSyncConfig {
                        filter: envelope_filter.clone(),
                    }),
                    ..Default::default()
                }),
                flag: Some(email::flag::config::FlagConfig {
                    sync: Some(email::flag::sync::config::FlagSyncConfig {
                        permissions: self.flag.map(|c| c.permissions).unwrap_or_default(),
                    }),
                    ..Default::default()
                }),
                message: Some(email::message::config::MessageConfig {
                    sync: Some(email::message::sync::config::MessageSyncConfig {
                        permissions: self.message.map(|c| c.permissions).unwrap_or_default(),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
    }
}

/// The backend-specific configuration.
///
/// Represents all valid backends managed by Neverest with their
/// specific configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum BackendConfig {
    /// The IMAP backend configuration.
    #[cfg(feature = "imap")]
    Imap(ImapConfig),

    /// The Maildir backend configuration.
    #[cfg(feature = "maildir")]
    Maildir(MaildirConfig),

    /// The Notmuch backend configuration.
    #[cfg(feature = "notmuch")]
    Notmuch(NotmuchConfig),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct FolderBackendConfig {
    #[serde(default)]
    pub permissions: FolderSyncPermissions,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct FlagBackendConfig {
    #[serde(default)]
    pub permissions: FlagSyncPermissions,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MessageBackendConfig {
    #[serde(default)]
    pub permissions: MessageSyncPermissions,
}