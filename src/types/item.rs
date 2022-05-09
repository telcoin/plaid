use serde::{Deserialize, Serialize};

/// Description of the kind of webhook
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "webhook_code")]
pub enum ItemWebhookCode {
    /// Fired when an error is encountered with an Item. The error can be resolved by having the user go through Link’s update mode.
    Error,
    /// Fired when Plaid detects a new account for Items created or updated with Account Select v2. Upon receiving this webhook, you can prompt your users to share new accounts with you through Account Select v2 update mode.
    NewAccountsAvailable,
    /// Fired when an Item’s access consent is expiring in 7 days. Some Items have explicit expiration times and we try to relay this when possible to reduce service disruption. This can be resolved by having the user go through Link’s update mode.
    PendingExpiration {
        /// The date and time at which the Item's access consent will expire, in ISO 8601 format
        consent_expiration_time: String,
    },
    /// The USER_PERMISSION_REVOKED webhook is fired to when an end user has used the my.plaid.com portal to revoke the permission that they previously granted to access an Item. Once access to an Item has been revoked, it cannot be restored. If the user subsequently returns to your application, a new Item must be created for the user.
    UserPermissionRevoked,
    /// Fired when an Item's webhook is updated. This will be sent to the newly specified webhook.
    WebhookUpdateAcknowledged {
        /// The new webhook URL
        new_webhook_url: String,
    },
}

/// Webhooks are used to communicate changes to an `Item`, such as an updated webhook, or errors encountered with an `Item`. The error typically requires user action to resolve, such as when a user changes their password. All `Item` webhooks have a `webhook_type` of `ITEM`.
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemWebhook {
    /// Description of the kind of webhook
    #[serde(flatten)]
    pub webhook_code: ItemWebhookCode,
    /// The item_id of the Item associated with this webhook, warning, or error
    pub item_id: String,
}
