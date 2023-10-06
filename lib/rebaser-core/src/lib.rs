//! This library exists to ensure that rebaser-client does not depend on rebaser-server and vice
//! versa. Keeping the dependency chain intact is important because rebaser-server depends on the
//! dal and the dal (really anyone) must be able to use the rebaser-client.
//!
//! This library also contains tests for rebaser-client and rebaser-server interaction.

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use ulid::Ulid;

/// The action for the rebaser management loop.
#[derive(Debug, Serialize, Deserialize)]
pub enum ManagementMessageAction {
    /// Close the inner rebaser loop for a change set. If it has already been closed, this is a
    /// no-op.
    CloseChangeSet,
    /// Open the inner rebaser loop for a change set. If one already exists, it is a no-op.
    OpenChangeSet,
}

/// The message that the rebaser management consumer expects in the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct ManagementMessage {
    /// The ID of the change set wishing to be operated on.
    pub change_set_id: Ulid,
    /// The action to instruct the management loop to perform.
    pub action: ManagementMessageAction,
}

/// The message that the server's listener loop uses to perform a rebase.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeSetMessage {
    /// Corresponds to the change set whose pointer is to be updated.
    pub to_rebase_change_set_id: Ulid,
    /// Corresponds to the workspace snapshot that will be the "onto" workspace snapshot when
    /// rebasing the "to rebase" workspace snapshot.
    pub onto_workspace_snapshot_id: Ulid,
    /// Derived from the ephemeral or persisted change set that's either the base change set, the
    /// last change set before edits were made, or the change set that you are trying to rebase
    /// onto base.
    pub onto_vector_clock_id: Ulid,
}

/// The message shape that the rebaser change set loop will use for replying to the client.
#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeSetReplyMessage {
    /// Processing the delivery and performing updates was successful.
    Success {
        /// The serialized updates performed when rebasing.
        updates_performed: Value,
    },
    /// Conflicts found when processing the delivery.
    ConflictsFound {
        /// A serialized list of the conflicts found during detection.
        conflicts_found: Value,
        /// A serialized list of the updates found during detection and skipped because at least
        /// once conflict was found.
        updates_found_and_skipped: Value,
    },
    /// Error encountered when processing the delivery.
    Error {
        /// The error message.
        message: String,
    },
}

/// A generator that provides stream names in a centralized location.
#[allow(missing_debug_implementations)]
pub struct StreamNameGenerator;

impl StreamNameGenerator {
    /// Returns the name of the management stream.
    pub fn management() -> &'static str {
        "rebaser-management"
    }

    /// Returns the name of the stream that the rebaser will reply to for messages sent to the
    /// management stream from a specific client.
    pub fn management_reply(client_id: Ulid) -> String {
        format!("rebaser-management-reply-{client_id}")
    }

    /// Returns the name of a stream for a given change set.
    pub fn change_set(change_set_id: Ulid) -> String {
        format!("rebaser-{change_set_id}")
    }

    /// Returns the name of the stream that the rebaser will reply to for messages sent to a change
    /// set stream from a specific client.
    pub fn change_set_reply(change_set_id: Ulid, client_id: Ulid) -> String {
        format!("rebaser-{change_set_id}-reply-{client_id}")
    }
}
