use idb::DatabaseEvent;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("Failed to create IDB factory: {0}")]
    Idb(#[from] idb::Error),
    #[error("LWK error: {0}")]
    Lwk(#[from] lwk_wollet::PersistError),
    #[error("Failed to serialize/deserialize update: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),
}

const DB_NAME: &str = "nostrades";
const UPDATES_STORE_NAME: &str = "updates";
const TIP_KEY: &str = "tip";

#[derive(Debug, Clone)]
pub struct IdbPersister {
    db: std::rc::Rc<idb::Database>,
}

impl IdbPersister {
    fn upgrade_needed(event: idb::event::VersionChangeEvent) -> Result<(), PersistError> {
        let db = event.database()?;
        // Prepare object store params
        let mut store_params = idb::ObjectStoreParams::new();
        store_params.auto_increment(true);
        store_params.key_path(Some(idb::KeyPath::new_single(TIP_KEY)));

        // Create object store
        db.create_object_store(UPDATES_STORE_NAME, store_params)?;
        Ok(())
    }

    pub async fn new() -> Result<Self, PersistError> {
        let factory = idb::Factory::new()?;

        // Create an open request for the database
        let mut open_request = factory.open(DB_NAME, Some(1))?;

        // Set up the upgrade needed event
        open_request.on_upgrade_needed(|event| {
            if let Err(e) = Self::upgrade_needed(event) {
                web_sys::console::error_1(&format!("Failed to upgrade IDB: {e}").into());
            }
        });

        let db = open_request.await?;
        Ok(IdbPersister {
            db: std::rc::Rc::new(db),
        })
    }
    pub async fn push(&self, update: lwk_wollet::Update) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;
        let idb_update: IdbUpdate = update.try_into()?;
        store.add(&serde_wasm_bindgen::to_value(&idb_update)?, None)?;
        tx.commit()?;
        Ok(())
    }
    pub async fn get_all(&self) -> Result<Vec<Update>, PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;

        let stored_updates: Vec<wasm_bindgen::JsValue> = store.get_all(None, None).unwrap().await?;

        Ok(stored_updates
            .into_iter()
            .filter_map(|value| {
                let idb_update: Result<IdbUpdate, _> = serde_wasm_bindgen::from_value(value);
                idb_update.ok()?.try_into().ok()
            })
            .collect())
    }
}

use lwk_wollet::Update;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdbUpdate {
    tip: u32,
    data: Vec<u8>,
}
impl TryFrom<Update> for IdbUpdate {
    type Error = lwk_wollet::PersistError;

    fn try_from(update: Update) -> Result<Self, Self::Error> {
        Ok(IdbUpdate {
            tip: update.tip.height,
            data: update.serialize()?,
        })
    }
}
impl TryFrom<IdbUpdate> for Update {
    type Error = lwk_wollet::PersistError;

    fn try_from(idb_update: IdbUpdate) -> Result<Self, Self::Error> {
        let update = lwk_wollet::Update::deserialize(&idb_update.data)?;
        if update.tip.height != idb_update.tip {
            return Err(lwk_wollet::PersistError::Other(
                "Tip height mismatch".into(),
            ));
        }
        Ok(update)
    }
}
