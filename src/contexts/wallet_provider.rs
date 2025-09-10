use yew::prelude::*;

pub static ESPLORA_CLIENT: std::sync::LazyLock<
    tokio::sync::RwLock<lwk_wollet::clients::asyncr::EsploraClient>,
> = std::sync::LazyLock::new(|| {
    lwk_wollet::clients::asyncr::EsploraClientBuilder::new(
        "https://liquid.network/liquidtestnet/api/",
        lwk_wollet::ElementsNetwork::LiquidTestnet,
    )
    .timeout(3)
    .build()
    .expect("Failed to create BTC Esplora client")
    .into()
});

#[derive(Debug, thiserror::Error)]
pub enum NostradeWalletError {
    #[error("LWK error: {0}")]
    Lwk(#[from] lwk_wollet::Error),
    #[error("LWK sign error: {0}")]
    LwkSign(#[from] lwk_signer::SignError),
    #[error("LWK persist error: {0}")]
    LwkPersist(#[from] lwk_wollet::PersistError),
    #[error("Persist error: {0}")]
    Persist(#[from] crate::persister::PersistError),
    #[error("No IDB")]
    NoPersister,
    #[error("No UTXO found")]
    NoUtxo,
}

#[derive(Clone, Debug)]
pub struct NostradeWallet {
    loaded: bool,
    synced: u64,
    wollet: std::sync::Arc<tokio::sync::RwLock<lwk_wollet::Wollet>>,
    signer: lwk_signer::SwSigner,
    persistor: Option<crate::persister::IdbPersister>,
}
impl NostradeWallet {
    pub const fn persistor(&self) -> Option<&crate::persister::IdbPersister> {
        self.persistor.as_ref()
    }
    pub const fn loaded(&self) -> bool {
        self.loaded
    }
    pub const fn synced(&self) -> bool {
        self.synced > 0
    }
    pub async fn load(&self) -> Result<(), NostradeWalletError> {
        web_sys::console::log_1(&"Loading wallet...".into());
        let mut wollet = self.wollet.write().await;
        let Some(persistor) = &self.persistor else {
            return Ok(());
        };
        let updates = persistor.get_all_updates().await?;

        for update in updates {
            if let Err(e) = wollet.apply_update_no_persist(update.clone()) {
                web_sys::console::error_1(&format!("Failed to apply update: {e}").into());
            }
        }
        web_sys::console::log_1(&"Wallet loaded".into());

        Ok(())
    }
    pub async fn simple_sync(&self) -> Result<(), NostradeWalletError> {
        let Some(persistor) = &self.persistor else {
            web_sys::console::error_1(&"No persistor found".into());
            return Err(NostradeWalletError::NoPersister);
        };
        let client = ESPLORA_CLIENT.write().await;
        let mut wollet = self.wollet.write().await;
        loop {
            let last_unused = wollet.address(None)?.address().script_pubkey();
            let history = client
                .get_scripts_history(&[&last_unused])
                .await?
                .iter()
                .flatten()
                .map(|tx| tx.txid)
                .collect::<Vec<_>>();
            if history.is_empty() {
                break;
            }
            let transactions = client.get_transactions(&history).await?;
            for tx in transactions {
                if wollet.apply_transaction(tx).is_err() {
                    web_sys::console::error_1(&"Failed to apply transaction".into());
                }
            }
            let updates = wollet.updates()?;
            for update in updates {
                if let Err(e) = persistor.push_update(update.clone()).await {
                    web_sys::console::error_1(&format!("Failed to persist update: {e}").into());
                }
            }
        }
        drop(client);
        drop(wollet);
        Ok(())
    }
    pub async fn full_sync(&self) -> Result<(), NostradeWalletError> {
        let Some(persistor) = &self.persistor else {
            web_sys::console::error_1(&"No persistor found".into());
            return Err(NostradeWalletError::NoPersister);
        };
        let mut client = ESPLORA_CLIENT.write().await;
        let mut wollet = self.wollet.write().await;

        let Some(update) = client.full_scan(&wollet).await? else {
            return Ok(());
        };
        drop(client);
        persistor.push_update(update.clone()).await?;
        wollet.apply_update_no_persist(update)?;
        drop(wollet);
        Ok(())
    }
    pub async fn balance(
        &self,
    ) -> Result<std::collections::BTreeMap<elements::AssetId, u64>, NostradeWalletError> {
        Ok(self.wollet.read().await.balance()?)
    }
    pub async fn address(&self) -> Result<elements::Address, NostradeWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.address(None).map(|addr| addr.address().clone())?)
    }
    pub async fn utxos(&self) -> Result<Vec<lwk_wollet::WalletTxOut>, NostradeWalletError> {
        let Some(persistor) = &self.persistor else {
            return Err(NostradeWalletError::NoPersister);
        };
        let wollet = self.wollet.read().await;
        let all_utxos = wollet.utxos()?;
        drop(wollet);
        let contracted_utxo_ids = persistor.get_proposal_utxos().await?;
        let available_utxos = all_utxos
            .into_iter()
            .filter(|utxo| !contracted_utxo_ids.contains(&utxo.outpoint.txid))
            .collect::<Vec<_>>();
        Ok(available_utxos)
    }
    pub async fn transactions(&self) -> Result<Vec<lwk_wollet::WalletTx>, NostradeWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.transactions()?)
    }

    pub async fn send_coins(
        &self,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<elements::Txid, NostradeWalletError> {
        let Some(persistor) = &self.persistor else {
            return Err(NostradeWalletError::NoPersister);
        };
        let wollet = self.wollet.write().await;
        let contracted_utxo_ids = persistor.get_proposal_utxos().await?;

        let available_utxos = wollet
            .utxos()?
            .into_iter()
            .filter_map(|utxo| {
                (!contracted_utxo_ids.contains(&utxo.outpoint.txid)).then_some(utxo.outpoint)
            })
            .collect::<Vec<_>>();

        let mut pset = wollet
            .tx_builder()
            .set_wallet_utxos(available_utxos)
            // TODO: PROPER UTXO SELECTION
            .add_recipient(recipient, amount, asset_id)?
            .fee_rate(Some(100.)) // Adjust fee rate as needed
            .finish()?;
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        let tx = wollet.finalize(&mut pset)?;
        let tx_id = ESPLORA_CLIENT.write().await.broadcast(&tx).await?;
        let updates = wollet.updates()?;

        for update in updates {
            if let Err(e) = persistor.push_update(update.clone()).await {
                web_sys::console::error_1(&format!("Failed to persist update: {e}").into());
            }
        }
        drop(wollet);
        Ok(tx_id)
    }

    pub async fn sized_liquidex_proposal(
        &self,
        input_asset: elements::AssetId,
        desired_size: u64,
        output_asset: elements::AssetId,
        swap_amount: u64,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, NostradeWalletError> {
        let address = self.address().await?;
        let tx = self.send_coins(&address, desired_size, input_asset).await?;
        let client = ESPLORA_CLIENT.write().await;
        let mut wollet = self.wollet.write().await;
        let tx_details = client.get_transactions(&[tx]).await?;
        drop(client);
        for tx in tx_details {
            wollet.apply_transaction(tx)?;
        }
        web_sys::console::log_1(&format!("Applied tx: {tx}").into());
        drop(wollet);

        let matching_utxo = self
            .utxos()
            .await?
            .into_iter()
            .find(|utxo| utxo.unblinded.value == desired_size)
            .ok_or(NostradeWalletError::NoUtxo)?;
        let pset = self
            .liquidex_proposal(matching_utxo.outpoint, &address, swap_amount, output_asset)
            .await?;
        web_sys::console::log_1(&format!("Found UTXO: {matching_utxo:?}").into());

        Ok(pset)
    }

    pub async fn liquidex_proposal(
        &self,
        utxo: elements::OutPoint,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, NostradeWalletError> {
        let wollet = self.wollet.read().await;
        let mut pset = wollet
            .tx_builder()
            .liquidex_make(utxo, recipient, amount, asset_id)?
            // .add_recipient("<ILLUMINODES ADDRESS>", 1000, "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49")?
            .finish()?;
        drop(wollet);
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        Ok(lwk_wollet::LiquidexProposal::from_pset(&pset)?)
    }
    pub async fn liquidex_take(
        &self,
        proposal: lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
    ) -> Result<(), NostradeWalletError> {
        let wollet = self.wollet.read().await;
        let mut pset = wollet
            .tx_builder()
            .liquidex_take(vec![proposal])?
            // .add_recipient("<ILLUMINODES ADDRESS>", 1000, "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49")?
            .finish()?;
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        let tx = wollet.finalize(&mut pset)?;
        let _tx_id = ESPLORA_CLIENT.write().await.broadcast(&tx).await?;
        let Some(persistor) = &self.persistor else {
            return Err(NostradeWalletError::NoPersister);
        };
        let updates = wollet.updates()?;
        drop(wollet);

        for update in updates {
            if let Err(e) = persistor.push_update(update.clone()).await {
                web_sys::console::error_1(&format!("Failed to persist update: {e}").into());
            }
        }
        Ok(())
    }
}

impl PartialEq for NostradeWallet {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

pub enum NostradeWalletAction {
    Loaded,
    Synced,
}

impl Reducible for NostradeWallet {
    type Action = NostradeWalletAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            NostradeWalletAction::Loaded => {
                web_sys::console::log_1(&"Wallet loaded".into());
                std::rc::Rc::new(Self {
                    loaded: true,
                    synced: self.synced,
                    wollet: self.wollet.clone(),
                    persistor: self.persistor.clone(),
                    signer: self.signer.clone(),
                })
            }
            NostradeWalletAction::Synced => {
                web_sys::console::log_1(&"Wallet synced".into());
                std::rc::Rc::new(Self {
                    loaded: self.loaded,
                    synced: self.synced + 1,
                    wollet: self.wollet.clone(),
                    persistor: self.persistor.clone(),
                    signer: self.signer.clone(),
                })
            }
        }
    }
}

pub type NostradeWalletStore = UseReducerHandle<NostradeWallet>;

#[function_component(WalletProvider)]
pub fn language_config_provider(props: &yew::html::ChildrenProps) -> HtmlResult {
    let Some(mut nostr_key) = nostr_minions::use_nostr_key() else {
        return Ok(html! {});
    };
    nostr_key.set_extractable(true);
    let mnemonic = nostr_key
        .mnemonic(nostr_minions::nostro2_signer::Language::English)
        .expect("Failed to get mnemonic");
    nostr_key.set_extractable(false);
    let signer = lwk_signer::SwSigner::new(&mnemonic, false).expect("Failed to create signer");

    let wollet = lwk_wollet::Wollet::without_persist(
        lwk_wollet::ElementsNetwork::LiquidTestnet,
        lwk_common::singlesig_desc(
            &signer,
            lwk_common::Singlesig::Wpkh,
            lwk_common::DescriptorBlindingKey::Slip77,
        )
        .expect("Failed to create descriptor")
        .parse()
        .expect("Failed to parse descriptor"),
    )
    .expect("Failed to create Wollet");

    let persistor =
        yew::suspense::use_future(|| async { crate::persister::IdbPersister::new().await })?;

    let ctx = use_reducer(|| NostradeWallet {
        loaded: false,
        synced: 0,
        signer,
        persistor: persistor.as_ref().ok().cloned(),
        wollet: std::sync::Arc::new(tokio::sync::RwLock::new(wollet)),
    });

    Ok(html! {
        <ContextProvider<NostradeWalletStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<NostradeWalletStore>>
    })
}

#[hook]
pub fn use_wallet_ctx() -> NostradeWalletStore {
    use_context::<NostradeWalletStore>().expect("No wallet context found")
}

#[hook]
pub fn use_wallet_address() -> Option<elements::Address> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    let address = yew::suspense::use_future_with(
        (wallet_ctx.loaded, wallet_ctx.synced),
        |loaded| async move {
            if !loaded.0 {
                return Err(());
            }
            match ctx_clone.address().await {
                Ok(address) => Ok(address),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get address: {e}").into());
                    Err(())
                }
            }
        },
    )
    .ok()?;
    (*address).clone().ok()
}

#[hook]
pub fn use_wallet_balance() -> Result<
    yew::suspense::UseFutureHandle<std::collections::BTreeMap<elements::AssetId, u64>>,
    yew::suspense::Suspension,
> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(
        (wallet_ctx.loaded, wallet_ctx.synced),
        |loaded| async move {
            if !loaded.0 {
                return std::collections::BTreeMap::new();
            }
            match ctx_clone.balance().await {
                Ok(balance) => balance,
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get balance: {e}").into());
                    std::collections::BTreeMap::new()
                }
            }
        },
    )
}

#[hook]
pub fn use_wallet_utxos() -> Result<Vec<lwk_wollet::WalletTxOut>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    let utxos = yew::suspense::use_future_with(
        (wallet_ctx.loaded, wallet_ctx.synced),
        |loaded| async move {
            if !loaded.0 {
                return Ok::<Vec<_>, ()>(vec![]);
            }
            match ctx_clone.utxos().await {
                Ok(utxos) => Ok::<_, ()>(utxos),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get UTXOs: {e}").into());
                    Ok(vec![])
                }
            }
        },
    )?;
    Ok((*utxos).clone().unwrap_or(vec![]))
}

#[hook]
pub fn use_wallet_transactions() -> Result<Vec<lwk_wollet::WalletTx>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    let transactions = yew::suspense::use_future_with(
        (wallet_ctx.loaded, wallet_ctx.synced),
        |loaded| async move {
            if !loaded.0 {
                return Ok::<Vec<_>, ()>(vec![]);
            }
            match ctx_clone.transactions().await {
                Ok(transactions) => Ok::<_, ()>(transactions),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get transactions: {e}").into());
                    Ok(vec![])
                }
            }
        },
    )?;
    Ok((*transactions).clone().unwrap_or(vec![]))
}
