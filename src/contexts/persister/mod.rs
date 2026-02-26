//! # Persister
//!
//! IndexedDB-backed persistence layer for the SwappingSats browser wallet.
//!
//! The database is named `"nostrades"` and contains three object stores:
//!
//! | Store | Key | Purpose |
//! |-------|-----|---------|
//! | `updates` | `tip` (block height) | LWK wallet state snapshots. Loaded on startup so the wallet can resume from its last known chain tip without a full rescan. |
//! | `locked_utxos` | auto-increment | UTXOs currently committed to an open swap offer.  Excluded from coin selection until the offer is accepted, cancelled, or the UTXO is no longer in the wallet. |
//! | `swaps` | Nostr event `id` | Records of accepted or failed swap attempts.  Used to filter duplicates from the order book UI. |

mod handler;
mod hooks;
mod provider;

pub use handler::*;
pub use hooks::*;
pub use provider::*;

/// Lifecycle state of a swap tracked in IndexedDB.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum SwapStatus {
    /// The taker successfully broadcast the swap transaction.
    Accepted,
    /// The attempt to take the swap failed (e.g. UTXO already spent).
    Failed,
    /// The offer was manually filtered by the user and should not appear again.
    Filtered,
}

/// A swap record persisted in the `swaps` IndexedDB store.
///
/// `id` is the Nostr event ID of the offer note.  It serves as the primary key
/// so the same offer can never be accepted or filtered more than once, even
/// across page reloads.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedSwap {
    /// Nostr event ID of the offer (hex string, used as IDB primary key).
    pub id: String,
    pub status: SwapStatus,
}
impl PersistedSwap {
    #[must_use]
    pub const fn new(id: String, status: SwapStatus) -> Self {
        Self { id, status }
    }
}

/// Serialisable wrapper around [`lwk_wollet::Update`] suitable for IndexedDB.
///
/// `tip` stores the block height so updates can be integrity-checked on
/// deserialisation.  `data` is the raw LWK serialisation of the update.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdbUpdate {
    tip: u32,
    data: Vec<u8>,
}
impl TryFrom<lwk_wollet::Update> for IdbUpdate {
    type Error = lwk_wollet::PersistError;

    fn try_from(update: lwk_wollet::Update) -> Result<Self, Self::Error> {
        Ok(Self {
            tip: update.tip.height,
            data: update.serialize()?,
        })
    }
}
impl TryFrom<IdbUpdate> for lwk_wollet::Update {
    type Error = lwk_wollet::PersistError;

    fn try_from(idb_update: IdbUpdate) -> Result<Self, Self::Error> {
        let update = Self::deserialize(&idb_update.data)?;
        if update.tip.height != idb_update.tip {
            return Err(lwk_wollet::PersistError::Other(
                "Tip height mismatch".into(),
            ));
        }
        Ok(update)
    }
}
