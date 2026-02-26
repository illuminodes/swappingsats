//! # Liquid Web Wallet
//!
//! Low-level interface to a Liquid Network wallet backed by the
//! [LWK (Liquid Wallet Kit)](https://github.com/Blockstream/lwk) library.
//!
//! ## Key responsibilities
//!
//! * **Wallet initialisation** – derives a WPKH descriptor with SLIP-77
//!   confidential blinding from a BIP-39 mnemonic (itself derived from the
//!   user's Nostr key inside the higher-level [`crate::WalletProvider`]).
//! * **Blockchain sync** – [`LiquidWebWallet::full_scan`] queries the Esplora
//!   API ([`ESPLORA_CLIENT`]) and applies the resulting
//!   [`lwk_wollet::Update`] to the in-memory wallet state.
//! * **Making a swap offer** – [`LiquidWebWallet::liquidex_proposal`] builds
//!   and half-signs a PSET for the *maker* side of a LiquiDEX atomic swap.
//! * **Taking a swap offer** – [`LiquidWebWallet::liquidex_take`] completes a
//!   validated proposal into a fully-signed transaction and broadcasts it.
//!
//! ## LiquiDEX atomic swap primer
//!
//! LiquiDEX is a protocol for non-custodial atomic swaps on the Liquid Network
//! using Partially Signed Element Transactions (PSETs):
//!
//! 1. **Maker** calls [`liquidex_proposal`][LiquidWebWallet::liquidex_proposal]:
//!    * Builds a PSET with their UTXO as input and the desired output (asset +
//!      amount) directed back to their own address.
//!    * Signs only their own input (half-signed PSET).
//!    * Serialises the result as a [`lwk_wollet::LiquidexProposal<Unvalidated>`].
//!    * The JSON of this proposal becomes the `content` of a Nostr kind-`32121`
//!      event that is broadcast to the relay pool.
//!
//! 2. **Taker** calls [`liquidex_take`][LiquidWebWallet::liquidex_take]:
//!    * Receives a [`lwk_wollet::LiquidexProposal<Validated>`] (the proposal
//!      has been verified against the blockchain by [`crate::OrderBook::parsed_offers`]).
//!    * Adds their own UTXOs to the PSET to satisfy the maker's requested output.
//!    * Signs their inputs.
//!    * Finalises and broadcasts the complete transaction via [`ESPLORA_CLIENT`].
//!    * Returns the resulting `Txid`.

pub static FEE_ADDRESS: std::sync::LazyLock<elements::Address> = std::sync::LazyLock::new(|| {
    "tlq1qqv8caryh8kdy6v3mgn6cljngks9geedrcdsxa8eav5l2p8hmcz3kedv082nkdurnjta8rrt2wjlhgk86mlhk5r2tjt0hkp4ty"
            .parse::<elements::Address>()
            .expect("Failed to parse address")
});

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
    /// # Errors
    /// Returns an error if signer creation, descriptor parsing, or wallet initialization fails.
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
    /// # Errors
    /// Returns an error if the wallet balance cannot be retrieved.
    pub async fn balance(
        &self,
    ) -> Result<std::collections::BTreeMap<elements::AssetId, u64>, LiquidWebWalletError> {
        Ok(self.wollet.read().await.balance()?)
    }
    /// # Errors
    /// Returns an error if the wallet address cannot be generated.
    pub async fn address(&self) -> Result<elements::Address, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.address(None).map(|addr| addr.address().clone())?)
    }
    /// # Errors
    /// Returns an error if the wallet UTXOs cannot be retrieved.
    pub async fn utxos(&self) -> Result<Vec<lwk_wollet::WalletTxOut>, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.utxos()?)
    }
    /// # Errors
    /// Returns an error if the wallet transactions cannot be retrieved.
    pub async fn transactions(&self) -> Result<Vec<lwk_wollet::WalletTx>, LiquidWebWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.transactions()?)
    }
    /// # Errors
    /// Returns an error if transaction building, signing, finalization, or broadcasting fails.
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
    /// Constructs the **maker side** of a LiquiDEX atomic swap.
    ///
    /// Builds a PSET where:
    /// * **Input**: the caller's UTXO at `utxo` (the asset being offered).
    /// * **Output**: `amount` of `asset_id` sent back to the caller's own
    ///   `recipient` address (the asset being requested).
    ///
    /// The PSET is signed with the wallet's software signer and converted to a
    /// [`lwk_wollet::LiquidexProposal<Unvalidated>`].  The taker side remains
    /// unsigned; the proposal is only half-complete at this point.
    ///
    /// The returned proposal must be serialised to JSON and published to the
    /// Nostr relay pool as a kind-`32121` event (see [`crate::pages::SwapTesting`]).
    ///
    /// # Errors
    /// Returns an error if proposal creation, signing, or PSET conversion fails.
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

    /// Completes the **taker side** of a LiquiDEX atomic swap and broadcasts it.
    ///
    /// Takes a proposal that has already been validated against the blockchain
    /// (see [`crate::OrderBook::parsed_offers`]) and:
    ///
    /// 1. Builds a new PSET that includes the validated proposal plus the
    ///    taker's own `utxos` (to fund the maker's requested output).
    /// 2. Signs the taker's inputs.
    /// 3. Finalises the PSET into a complete Liquid transaction.
    /// 4. Broadcasts it to the network via [`ESPLORA_CLIENT`].
    ///
    /// On success the swap is atomic: both parties' assets are exchanged in a
    /// single on-chain transaction with no trusted third party.
    ///
    /// # Errors
    /// Returns an error if transaction building, signing, finalization, or broadcasting fails.
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

    /// # Errors
    /// Returns an error if wallet update application fails.
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

    /// # Errors
    /// Returns an error if the full scan operation or update application fails.
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
