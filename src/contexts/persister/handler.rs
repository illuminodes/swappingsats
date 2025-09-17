const DB_NAME: &str = "nostrades";

/// Stores the wallet state
const UPDATES_STORE_NAME: &str = "updates";
/// Stores utxos that have been offered for a swap
const LOCKED_UTXOS_STORE_NAME: &str = "locked_utxos";
/// Stores the swap ids that have been accepted or filtered out
/// No need to store confirmation details as blockchain should be source of truth
const SWAPS_STORE_NAME: &str = "swaps";

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("Failed to create IDB factory: {0}")]
    Idb(#[from] idb::Error),
    #[error("LWK persistance error: {0}")]
    LwkPersistance(#[from] lwk_wollet::PersistError),
    #[error("Failed to serialize/deserialize update: {0}")]
    WasmSerde(#[from] serde_wasm_bindgen::Error),
}

#[derive(Debug, Clone)]
pub struct NostradesIdb {
    db: std::rc::Rc<idb::Database>,
}
impl PartialEq for NostradesIdb {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl NostradesIdb {
    fn upgrade_needed(event: &idb::event::VersionChangeEvent) -> Result<(), PersistError> {
        let db = idb::event::DatabaseEvent::database(event)?;

        // Wallet state (single tip)
        let mut store_params = idb::ObjectStoreParams::new();
        store_params.key_path(Some(idb::KeyPath::new_single("tip")));
        db.create_object_store(UPDATES_STORE_NAME, store_params)?;

        // Locked UTXOs (we always fetch all)
        let mut store_params = idb::ObjectStoreParams::new();
        store_params.auto_increment(true); // Key can be auto-incremented, as we always need to recover all utxos
        db.create_object_store(LOCKED_UTXOS_STORE_NAME, store_params)?;

        // Swaps (keyed by swap Nostr id)
        let mut store_params = idb::ObjectStoreParams::new();
        store_params.key_path(Some(idb::KeyPath::new_single("id")));
        db.create_object_store(SWAPS_STORE_NAME, store_params)?;

        Ok(())
    }

    /// Creates a new `NostradesIdb` instance
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the `IndexedDB` factory cannot be created,
    /// the database cannot be opened, or the database upgrade fails.
    pub async fn new() -> Result<Self, PersistError> {
        let factory = idb::Factory::new()?;

        // Create an open request for the database
        let mut open_request = factory.open(DB_NAME, Some(2))?;

        // Set up the upgrade needed event
        open_request.on_upgrade_needed(|event| {
            if let Err(e) = Self::upgrade_needed(&event) {
                web_sys::console::error_1(&format!("Failed to upgrade IDB: {e}").into());
            }
        });

        let db = open_request.await?;
        Ok(Self {
            db: std::rc::Rc::new(db),
        })
    }
    /// Stores a wallet update to the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails or
    /// `PersistError::WasmSerde` if serialization fails.
    pub async fn push_update(&self, update: lwk_wollet::Update) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;
        let idb_update: super::IdbUpdate = update.try_into()?;
        store
            .add(&serde_wasm_bindgen::to_value(&idb_update)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    /// Retrieves all stored wallet updates from the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails.
    pub async fn get_all_updates(&self) -> Result<Vec<lwk_wollet::Update>, PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;

        let stored_updates: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_updates
            .into_iter()
            .filter_map(|value| {
                let idb_update: Result<super::IdbUpdate, _> = serde_wasm_bindgen::from_value(value);
                idb_update.ok()?.try_into().ok()
            })
            .collect())
    }

    /// Stores a locked UTXO to the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails or
    /// `PersistError::WasmSerde` if serialization fails.
    pub async fn push_locked_utxo(&self, utxo: elements::OutPoint) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[LOCKED_UTXOS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(LOCKED_UTXOS_STORE_NAME)?;
        store
            .put(&serde_wasm_bindgen::to_value(&utxo)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    /// Retrieves all locked UTXOs from the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails.
    pub async fn get_all_locked_utxos(&self) -> Result<Vec<elements::OutPoint>, PersistError> {
        let tx = self
            .db
            .transaction(&[LOCKED_UTXOS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(LOCKED_UTXOS_STORE_NAME)?;

        let stored_utxos: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_utxos
            .into_iter()
            .filter_map(|value| {
                let utxo: Result<elements::OutPoint, _> = serde_wasm_bindgen::from_value(value);
                utxo.ok()
            })
            .collect())
    }
    /// Removes UTXOs from the locked store that are not in the current outpoints list
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails or
    /// `PersistError::WasmSerde` if deserialization fails.
    pub async fn unlock_utxos(
        &self,
        current_outpoints: &[elements::OutPoint],
    ) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[LOCKED_UTXOS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(LOCKED_UTXOS_STORE_NAME)?;
        let mut cursor = store.open_cursor(None, None)?.await?;
        while let Some(c) = cursor {
            // Assuming value is stored as OutPoint { txid, vout }
            let val: elements::OutPoint =
                serde_wasm_bindgen::from_value::<elements::OutPoint>(c.value()?)?;

            if !current_outpoints.contains(&val) {
                c.delete()?;
            }
            cursor = c.next(None)?.await?;
        }

        tx.commit()?.await?;
        Ok(())
    }

    /// Stores a swap to the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails or
    /// `PersistError::WasmSerde` if serialization fails.
    pub async fn push_swap(&self, swap: super::PersistedSwap) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[SWAPS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(SWAPS_STORE_NAME)?;
        store
            .put(&serde_wasm_bindgen::to_value(&swap)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    /// Retrieves all stored swaps from the database
    ///
    /// # Errors
    ///
    /// Returns `PersistError::Idb` if the database transaction fails.
    pub async fn get_all_swaps(&self) -> Result<Vec<super::PersistedSwap>, PersistError> {
        let tx = self
            .db
            .transaction(&[SWAPS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(SWAPS_STORE_NAME)?;

        let stored_swaps: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_swaps
            .into_iter()
            .filter_map(|value| {
                let swap: Result<super::PersistedSwap, _> = serde_wasm_bindgen::from_value(value);
                swap.ok()
            })
            .collect())
    }
}
