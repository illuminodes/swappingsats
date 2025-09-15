pub static FEE_ADDRESS: std::sync::LazyLock<elements::Address> = std::sync::LazyLock::new(
    || {
        "tlq1qqv8caryh8kdy6v3mgn6cljngks9geedrcdsxa8eav5l2p8hmcz3kedv082nkdurnjta8rrt2wjlhgk86mlhk5r2tjt0hkp4ty"
            .parse::<elements::Address>()
            .expect("Failed to parse address")
    },
);

pub const LIQUID_NETWORK_API: &str = "https://liquid.network/liquidtestnet/api/";
pub const WATERFALLS_API: &str = "https://waterfalls.liquidwebwallet.org/liquidtestnet/api";

pub static ESPLORA_REST_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

pub static ESPLORA_CLIENT: std::sync::LazyLock<
    tokio::sync::RwLock<lwk_wollet::clients::asyncr::EsploraClient>,
> = std::sync::LazyLock::new(|| {
    lwk_wollet::clients::asyncr::EsploraClientBuilder::new(
        // "https://liquid.network/liquidtestnet/api/",
        "https://waterfalls.liquidwebwallet.org/liquidtestnet/api",
        lwk_wollet::ElementsNetwork::LiquidTestnet,
    )
    .waterfalls(true)
    .timeout(3)
    .build()
    .expect("Failed to create BTC Esplora client")
    .into()
});

#[derive(Debug, thiserror::Error)]
pub enum LiquidWebWalletError {
    #[error("LWK error: {0}")]
    Lwk(#[from] lwk_wollet::Error),
    #[error("LWK sign error: {0}")]
    LwkSign(#[from] lwk_signer::SignError),
    #[error("LWK persist error: {0}")]
    LwkPersist(#[from] lwk_wollet::PersistError),
    #[error("No IDB")]
    NoPersister,
    #[error("No UTXO found")]
    NoUtxo,
    #[error("No proposal found")]
    Pset(#[from] elements::pset::Error),
    #[error("Descriptor error: {0}")]
    Descriptor(String),
    #[error("New key error: {0}")]
    NewKey(#[from] lwk_signer::NewError),
}

#[derive(Clone)]
pub struct LiquidWebWallet {
    wollet: std::sync::Arc<tokio::sync::RwLock<lwk_wollet::Wollet>>,
    signer: lwk_signer::SwSigner,
}
impl LiquidWebWallet {
    pub fn new(
        mnemonic: &str,
        network: lwk_wollet::ElementsNetwork,
    ) -> Result<Self, LiquidWebWalletError> {
        let signer = lwk_signer::SwSigner::new(
            mnemonic,
            matches!(network, lwk_wollet::ElementsNetwork::Liquid),
        )?;
        Ok(Self {
            wollet: std::sync::Arc::new(tokio::sync::RwLock::new(
                lwk_wollet::Wollet::without_persist(
                    network,
                    lwk_common::singlesig_desc(
                        &signer,
                        lwk_common::Singlesig::Wpkh,
                        lwk_common::DescriptorBlindingKey::Slip77,
                    )
                    .map_err(LiquidWebWalletError::Descriptor)?
                    .parse()?,
                )?,
            )),
            signer,
        })
    }
    pub async fn balance(
        &self,
    ) -> Result<std::collections::BTreeMap<elements::AssetId, u64>, LiquidWebWalletError> {
        Ok(self.wollet.read().await.balance()?)
    }
    pub async fn address(&self) -> Result<elements::Address, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.address(None).map(|addr| addr.address().clone())?)
    }
    pub async fn utxos(&self) -> Result<Vec<lwk_wollet::WalletTxOut>, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.utxos()?)
    }
    pub async fn transactions(&self) -> Result<Vec<lwk_wollet::WalletTx>, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.transactions()?)
    }
    pub async fn send_coins(
        &self,
        utxos: Vec<elements::OutPoint>,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<elements::Txid, LiquidWebWalletError> {
        let wollet = self.wollet.write().await;
        let mut pset = wollet
            .tx_builder()
            .set_wallet_utxos(utxos)
            .add_recipient(recipient, amount, asset_id)?
            // TODO: Get fee rate from mempool api
            .fee_rate(Some(100.))
            .finish()?;
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        let tx = wollet.finalize(&mut pset)?;
        drop(wollet);
        let tx_id = ESPLORA_CLIENT.write().await.broadcast(&tx).await?;
        Ok(tx_id)
    }
    pub async fn liquidex_proposal(
        &self,
        utxo: elements::OutPoint,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        let mut pset = wollet
            .tx_builder()
            .liquidex_make(utxo, recipient, amount, asset_id)?
            // TODO: Figure out if we can add the fee to the proposal so the maker can pay it
            // Maker pays the platform fee
            // .add_recipient(&crate::ILLUMINODES_ADDRESS, 1000, *crate::T_L_BTC_ASSET_ID)?
            .finish()?;
        drop(wollet);
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        Ok(lwk_wollet::LiquidexProposal::from_pset(&pset)?)
    }

    pub async fn liquidex_take(
        &self,
        // TODO: Accept batch of proposals
        utxos: Vec<elements::OutPoint>,
        proposal: lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
    ) -> Result<elements::Txid, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        // `let mut fee_pset = wollet
        // `    .tx_builder()
        // `    .add_recipient(&ILLUMINODES_ADDRESS, 1000, *crate::T_L_BTC_ASSET_ID)?
        // `    .finish()?;
        // `lwk_common::Signer::sign(&self.signer, &mut fee_pset)?;
        let mut pset = wollet
            .tx_builder()
            .set_wallet_utxos(utxos)
            .liquidex_take(vec![proposal])?
            .finish()?;
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        // for input in fee_pset.inputs() {
        //     pset.add_input(input.clone());
        // }
        // for output in fee_pset.outputs() {
        //     pset.add_output(output.clone());
        // }
        let tx = wollet.finalize(&mut pset)?;
        drop(wollet);
        Ok(ESPLORA_CLIENT.write().await.broadcast(&tx).await?)
    }

    pub async fn apply_updates(
        &self,
        updates: Vec<lwk_wollet::Update>,
    ) -> Result<(), LiquidWebWalletError> {
        let mut wollet = self.wollet.write().await;
        for update in updates {
            if let Err(e) = wollet.apply_update_no_persist(update) {
                web_sys::console::error_1(&format!("Failed to apply update: {e}").into());
            }
        }
        Ok(())
    }

    pub async fn full_scan(&self) -> Result<Option<lwk_wollet::Update>, LiquidWebWalletError> {
        let mut wollet = self.wollet.write().await;
        let Some(update) = ESPLORA_CLIENT.write().await.full_scan(&wollet).await? else {
            return Ok(None);
        };
        wollet.apply_update_no_persist(update.clone())?;
        drop(wollet);
        Ok(Some(update))
    }
}
