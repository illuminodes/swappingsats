mod handler;
mod hooks;
mod provider;

pub use handler::*;
pub use hooks::*;
pub use provider::*;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Copy)]
pub enum SwapStatus {
    Accepted,
    Failed,
    Filtered,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistedSwap {
    pub id: String,
    pub status: SwapStatus,
}
impl PersistedSwap {
    pub fn new(id: String, status: SwapStatus) -> Self {
        Self { id, status }
    }
}

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
