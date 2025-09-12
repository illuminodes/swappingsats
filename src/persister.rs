use idb::DatabaseEvent;

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("Failed to create IDB factory: {0}")]
    Idb(#[from] idb::Error),
    #[error("LWK error: {0}")]
    Lwk(#[from] lwk_wollet::Error),
    #[error("LWK persistance error: {0}")]
    LwkPersistance(#[from] lwk_wollet::PersistError),
    #[error("Failed to serialize/deserialize update: {0}")]
    WasmSerde(#[from] serde_wasm_bindgen::Error),
    #[error("No UTXO ID found")]
    NoUtxoId,
    #[error("No proposals found")]
    NoProposals,
    #[error("Failed to parse UTXO ID: {0}")]
    TxParse(#[from] elements::hashes::hex::HexToArrayError),
    #[error("Failed to parse: {0}")]
    Serde(#[from] serde_json::Error),
}

const DB_NAME: &str = "nostrades";
const UPDATES_STORE_NAME: &str = "updates";
const PROPOSALS_STORE_NAME: &str = "proposals";
const OFFERS_STORE_NAME: &str = "offers";
const TIP_KEY: &str = "tip";
const PROPOSAL_ID_KEY: &str = "tx_id";

#[derive(Debug, Clone)]
pub struct IdbPersister {
    db: std::rc::Rc<idb::Database>,
}

impl IdbPersister {
    fn upgrade_needed(event: &idb::event::VersionChangeEvent) -> Result<(), PersistError> {
        let db = event.database()?;

        let mut store_params = idb::ObjectStoreParams::new();
        store_params.key_path(Some(idb::KeyPath::new_single(TIP_KEY)));
        db.create_object_store(UPDATES_STORE_NAME, store_params)?;

        let mut store_params = idb::ObjectStoreParams::new();
        store_params.key_path(Some(idb::KeyPath::new_single(PROPOSAL_ID_KEY)));
        db.create_object_store(PROPOSALS_STORE_NAME, store_params)?;

        let mut store_params = idb::ObjectStoreParams::new();
        store_params.key_path(Some(idb::KeyPath::new_single(PROPOSAL_ID_KEY)));
        let store = db.create_object_store(OFFERS_STORE_NAME, store_params)?;
        store.create_index(
            "created_at",
            idb::KeyPath::Single("offer.created_at".into()),
            None,
        )?;

        Ok(())
    }

    pub async fn new() -> Result<Self, PersistError> {
        let factory = idb::Factory::new()?;

        // Create an open request for the database
        let mut open_request = factory.open(DB_NAME, Some(1))?;

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
    pub async fn push_update(&self, update: lwk_wollet::Update) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;
        let idb_update: IdbUpdate = update.try_into()?;
        store
            .add(&serde_wasm_bindgen::to_value(&idb_update)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    pub async fn get_all_updates(&self) -> Result<Vec<Update>, PersistError> {
        let tx = self
            .db
            .transaction(&[UPDATES_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(UPDATES_STORE_NAME)?;

        let stored_updates: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_updates
            .into_iter()
            .filter_map(|value| {
                let idb_update: Result<IdbUpdate, _> = serde_wasm_bindgen::from_value(value);
                idb_update.ok()?.try_into().ok()
            })
            .collect())
    }
    pub async fn push_proposal(&self, proposal: PersistedProposal) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[PROPOSALS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(PROPOSALS_STORE_NAME)?;
        store
            .put(&serde_wasm_bindgen::to_value(&proposal)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    pub async fn get_proposal(
        &self,
        tx_id: elements::Txid,
    ) -> Result<PersistedProposal, PersistError> {
        let tx = self
            .db
            .transaction(&[PROPOSALS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(PROPOSALS_STORE_NAME)?;

        let Some(proposal) = store
            .get_key(wasm_bindgen::JsValue::from(tx_id.to_string().as_str()))?
            .await?
        else {
            return Err(PersistError::NoUtxoId);
        };
        Ok(serde_wasm_bindgen::from_value(proposal)?)
    }
    pub async fn get_proposal_utxos(&self) -> Result<Vec<elements::Txid>, PersistError> {
        let tx = self
            .db
            .transaction(&[PROPOSALS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(PROPOSALS_STORE_NAME)?;

        let proposals = store
            .get_all_keys(None, None)?
            .await?
            .into_iter()
            .filter_map(|key| serde_wasm_bindgen::from_value::<elements::Txid>(key).ok())
            .collect::<Vec<_>>();
        Ok(proposals)
    }
    pub async fn delete_proposal(&self, txid: elements::Txid) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[PROPOSALS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(PROPOSALS_STORE_NAME)?;
        store.delete(serde_wasm_bindgen::to_value(&txid)?)?.await?;
        tx.commit()?.await?;
        Ok(())
    }
    pub async fn get_all_proposals(&self) -> Result<Vec<PersistedProposal>, PersistError> {
        let tx = self
            .db
            .transaction(&[PROPOSALS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(PROPOSALS_STORE_NAME)?;

        let stored_proposals: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_proposals
            .into_iter()
            .filter_map(|value| {
                let proposal: Result<PersistedProposal, _> = serde_wasm_bindgen::from_value(value);
                proposal.ok()
            })
            .collect())
    }
    pub async fn push_offer(&self, offer: PersistedOffer) -> Result<(), PersistError> {
        let tx = self
            .db
            .transaction(&[OFFERS_STORE_NAME], idb::TransactionMode::ReadWrite)?;
        let store = tx.object_store(OFFERS_STORE_NAME)?;
        store
            .put(&serde_wasm_bindgen::to_value(&offer)?, None)?
            .await?;
        tx.commit()?.await?;
        Ok(())
    }
    pub async fn get_all_offers(&self) -> Result<Vec<PersistedOffer>, PersistError> {
        let tx = self
            .db
            .transaction(&[OFFERS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(OFFERS_STORE_NAME)?;

        let stored_offers: Vec<wasm_bindgen::JsValue> = store.get_all(None, None)?.await?;

        Ok(stored_offers
            .into_iter()
            .filter_map(|value| {
                let offer: Result<PersistedOffer, _> = serde_wasm_bindgen::from_value(value);
                offer.ok()
            })
            .collect())
    }
    pub async fn get_offers_in_last_hour(
        &self,
    ) -> Result<Vec<lwk_wollet::LiquidexProposal<lwk_wollet::Validated>>, PersistError> {
        let tx = self
            .db
            .transaction(&[OFFERS_STORE_NAME], idb::TransactionMode::ReadOnly)?;
        let store = tx.object_store(OFFERS_STORE_NAME)?;
        let last_hour = (web_sys::js_sys::Date::now() / 1000.0) - 3600.0;
        let key_range = idb::KeyRange::lower_bound(&wasm_bindgen::JsValue::from(last_hour), None)?;
        let index = store.index("created_at")?;
        let proposals = index
            .get_all(Some(idb::Query::KeyRange(key_range)), None)?
            .await?
            .into_iter()
            .filter_map(|value| {
                let idb_offer: Result<PersistedOffer, _> = serde_wasm_bindgen::from_value(value);
                idb_offer.ok()?.offer().ok()?.insecure_validate().ok()
            })
            .collect::<Vec<_>>();
        Ok(proposals)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedProposal {
    pub tx_id: elements::Txid,
    pub proposal: nostr_minions::nostro2::NostrNote,
}
impl PersistedProposal {
    pub fn new(proposal: nostr_minions::nostro2::NostrNote) -> Result<Self, PersistError> {
        let tx_id = proposal
            .tags
            .first_parameter()
            .ok_or(PersistError::NoUtxoId)?
            .parse::<elements::Txid>()?;

        proposal
            .content
            .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()?;

        Ok(Self { tx_id, proposal })
    }
}
impl TryFrom<&PersistedProposal> for lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated> {
    type Error = PersistError;
    fn try_from(proposal: &PersistedProposal) -> Result<Self, Self::Error> {
        Ok(proposal.proposal.content.parse::<Self>()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedOffer {
    pub tx_id: elements::Txid,
    pub offer: nostr_minions::nostro2::NostrNote,
}
impl PersistedOffer {
    pub fn new(offer: nostr_minions::nostro2::NostrNote) -> Result<Self, PersistError> {
        let tx_id = offer
            .tags
            .first_parameter()
            .ok_or(PersistError::NoUtxoId)?
            .parse()?;
        Ok(Self { tx_id, offer })
    }
    pub fn tx_id(&self) -> elements::Txid {
        self.tx_id
    }
    pub fn offer_note(&self) -> &nostr_minions::nostro2::NostrNote {
        &self.offer
    }
    pub fn offer(
        &self,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, PersistError> {
        Ok(self.offer.content.parse()?)
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
        Ok(Self {
            tip: update.tip.height,
            data: update.serialize()?,
        })
    }
}
impl TryFrom<IdbUpdate> for Update {
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
