use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Description of the kind of webhook
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "webhook_code")]
pub enum ItemWebhookCode {
    /// Fired when an error is encountered with an Item. The error can be resolved by having the user go through Link’s update mode.
    Error,
    /// Fired when an Item that was previously in an errored state is successfully updated, such as when a user goes through Link's update mode and repairs their login.
    LoginRepaired,
    /// Fired when Plaid detects a new account for Items created or updated with Account Select v2. Upon receiving this webhook, you can prompt your users to share new accounts with you through Account Select v2 update mode.
    NewAccountsAvailable,
    /// Fired when an Item is scheduled to be disconnected, either because the institution is being migrated to a new integration or because the institution's access token is expiring.
    PendingDisconnect {
        /// The reason the Item is scheduled to be disconnected
        reason: PendingDisconnectReason,
        /// The date and time at which the Item is scheduled to disconnect, in ISO 8601 format
        disconnect_time: String,
    },
    /// Fired when an Item’s access consent is expiring in 7 days. Some Items have explicit expiration times and we try to relay this when possible to reduce service disruption. This can be resolved by having the user go through Link’s update mode.
    PendingExpiration {
        /// The date and time at which the Item's access consent will expire, in ISO 8601 format
        consent_expiration_time: String,
    },
    /// Fired when an end user has revoked access to a single account, via the institution's portal or a similar mechanism.
    UserAccountRevoked {
        /// The external account ID of the affected account
        account_id: String,
    },
    /// The USER_PERMISSION_REVOKED webhook is fired to when an end user has used the my.plaid.com portal to revoke the permission that they previously granted to access an Item. Once access to an Item has been revoked, it cannot be restored. If the user subsequently returns to your application, a new Item must be created for the user.
    UserPermissionRevoked,
    /// Fired when an Item's webhook is updated. This will be sent to the newly specified webhook.
    WebhookUpdateAcknowledged {
        /// The new webhook URL
        new_webhook_url: String,
    },
}

/// The reason an `Item` is scheduled to be disconnected.
#[derive(Serialize, Deserialize, JsonSchema, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PendingDisconnectReason {
    /// The institution is being migrated to a new integration.
    InstitutionMigration,
    /// The institution's access token is expiring.
    InstitutionTokenExpiration,
}

/// The Plaid environment a webhook was sent from.
#[derive(Serialize, Deserialize, JsonSchema, Copy, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WebhookEnvironment {
    /// The Sandbox environment.
    Sandbox,
    /// The Production environment.
    Production,
}

/// Webhooks are used to communicate changes to an `Item`, such as an updated webhook, or errors encountered with an `Item`. The error typically requires user action to resolve, such as when a user changes their password. All `Item` webhooks have a `webhook_type` of `ITEM`.
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ItemWebhook {
    /// Description of the kind of webhook
    #[serde(flatten)]
    pub webhook_code: ItemWebhookCode,
    /// The item_id of the Item associated with this webhook, warning, or error
    pub item_id: String,
    /// The `user_id` associated with the Item, if the Item was created with a user token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// The Plaid environment the webhook was sent from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<WebhookEnvironment>,
}
