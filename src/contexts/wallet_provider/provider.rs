use yew::prelude::*;
#[derive(Debug, thiserror::Error)]
pub enum NostradeWalletError {
    #[error("Persist error: {0}")]
    Persist(#[from] crate::PersistError),
    #[error("No IDB")]
    NoPersister,
    #[error("Liquid web wallet error: {0}")]
    LiquidWebWallet(#[from] crate::LiquidWebWalletError),
    #[error("No UTXO")]
    NoUtxo,
}

#[derive(Clone)]
pub struct NostradeWalletState {
    synced: u64,
    wallet: crate::LiquidWebWallet,
}
impl NostradeWalletState {
    #[must_use]
    pub const fn synced(&self) -> bool {
        self.synced > 0
    }
    /// # Errors
    /// Returns an error if wallet updates cannot be retrieved, applied, or if UTXO operations fail.
    pub async fn load(&self, persistor: &crate::NostradesIdb) -> Result<(), NostradeWalletError> {
        let updates = persistor.get_all_updates().await?;
        self.wallet.apply_updates(updates).await?;
        let locked_utxos = self.locked_utxos(persistor).await?;
        let utxos = self
            .wallet
            .utxos()
            .await?
            .into_iter()
            .filter(|u| !locked_utxos.contains(u))
            .map(|u| u.outpoint)
            .collect::<Vec<_>>();
        persistor.unlock_utxos(&utxos).await?;
        Ok(())
    }
    /// # Errors
    /// Returns an error if wallet scanning fails, update persistence fails, or UTXO operations fail.
    pub async fn update_wallet(
        &self,
        persistor: &crate::NostradesIdb,
    ) -> Result<(), NostradeWalletError> {
        let Some(update) = self.wallet.full_scan().await? else {
            return Ok(());
        };
        persistor.push_update(update).await?;
        let locked_utxos = self.locked_utxos(persistor).await?;
        let utxos = self
            .wallet
            .utxos()
            .await?
            .into_iter()
            // Ignore locked utxos
            .filter(|u| !locked_utxos.contains(u))
            .map(|u| u.outpoint)
            .collect::<Vec<_>>();
        // Ensure that we unlock all utxos
        persistor.unlock_utxos(&utxos).await?;
        Ok(())
    }
    /// # Errors
    /// Returns an error if wallet UTXOs cannot be retrieved or locked UTXO data cannot be fetched.
    pub async fn available_utxos(
        &self,
        persistor: &crate::NostradesIdb,
    ) -> Result<Vec<lwk_wollet::WalletTxOut>, NostradeWalletError> {
        let all_utxos = self.wallet.utxos().await?;
        let contracted_utxo_ids = persistor.get_all_locked_utxos().await?;
        let available_utxos = all_utxos
            .into_iter()
            .filter(|utxo| !contracted_utxo_ids.contains(&utxo.outpoint))
            .collect::<Vec<_>>();
        Ok(available_utxos)
    }
    /// # Errors
    /// Returns an error if wallet UTXOs cannot be retrieved or locked UTXO data cannot be fetched.
    pub async fn locked_utxos(
        &self,
        persistor: &crate::NostradesIdb,
    ) -> Result<Vec<lwk_wollet::WalletTxOut>, NostradeWalletError> {
        let all_utxos = self.wallet.utxos().await?;
        let contracted_utxo_ids = persistor.get_all_locked_utxos().await?;
        let locked_utxos = all_utxos
            .into_iter()
            .filter(|utxo| contracted_utxo_ids.contains(&utxo.outpoint))
            .collect::<Vec<_>>();
        Ok(locked_utxos)
    }
    /// # Errors
    /// Returns an error if wallet address retrieval fails, liquidex proposal creation fails, or UTXO locking fails.
    pub async fn create_swap_offer(
        &self,
        utxo: elements::OutPoint,
        amount: u64,
        asset_id: elements::AssetId,
        persistor: &crate::NostradesIdb,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, NostradeWalletError> {
        let recipient = self.wallet.address().await?;
        let proposal = self
            .wallet
            .liquidex_proposal(utxo, &recipient, amount, asset_id)
            .await?;
        persistor.push_locked_utxo(utxo).await?;
        Ok(proposal)
    }

    /// # Errors
    /// Returns an error if available UTXOs cannot be retrieved, liquidex transaction fails, or swap persistence fails.
    pub async fn liquidex_take(
        &self,
        proposal: lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
        id: String,
        persistor: &crate::NostradesIdb,
    ) -> Result<elements::Txid, NostradeWalletError> {
        let available_utxos = self
            .available_utxos(persistor)
            .await?
            .into_iter()
            .map(|utxo| utxo.outpoint)
            .collect::<Vec<_>>();
        let txid = self.wallet.liquidex_take(available_utxos, proposal).await?;
        persistor
            .push_swap(crate::PersistedSwap::new(id, crate::SwapStatus::Accepted))
            .await?;
        Ok(txid)
    }
    /// # Errors
    /// Returns an error if available UTXOs cannot be retrieved, insufficient funds, or transaction sending fails.
    pub async fn normal_coin_send(
        &self,
        address: elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
        persistor: &crate::NostradesIdb,
    ) -> Result<elements::Txid, NostradeWalletError> {
        let mut utxos = self.available_utxos(persistor).await?;
        utxos.sort_by(|a, b| a.unblinded.value.cmp(&b.unblinded.value));
        // Accumulate utxos until we have enough
        let mut total = 0;
        let mut selected_utxos = Vec::new();
        loop {
            let Some(next_utxo) = utxos.iter().find(|utxo| utxo.unblinded.asset == asset_id) else {
                return Err(NostradeWalletError::NoUtxo);
            };
            total += next_utxo.unblinded.value;
            selected_utxos.push(next_utxo.clone());
            if total >= amount {
                break;
            }
        }
        // if no utxo is L-BTC, add smallest L-BTC utxo to pay for fees
        if !selected_utxos
            .iter()
            .any(|utxo| utxo.unblinded.asset == *crate::T_L_BTC_ASSET_ID)
        {
            let next_l_btc_utxo = utxos
                .iter()
                .filter(|utxo| utxo.unblinded.asset == *crate::T_L_BTC_ASSET_ID)
                .min_by(|a, b| a.unblinded.value.cmp(&b.unblinded.value))
                .ok_or(NostradeWalletError::NoUtxo)?;
            selected_utxos.push(next_l_btc_utxo.clone());
        }
        let utxos = selected_utxos
            .iter()
            .map(|utxo| utxo.outpoint)
            .collect::<Vec<_>>();

        let tx_id = self
            .wallet
            .send_coins(utxos, &address, amount, asset_id)
            .await?;
        Ok(tx_id)
    }
    /// # Errors
    /// Returns an error if available UTXOs cannot be retrieved, no suitable fee UTXO is found, or transaction sending fails.
    pub async fn hard_cancel_swap(
        &self,
        utxo: elements::OutPoint,
        persistor: &crate::NostradesIdb,
    ) -> Result<(), NostradeWalletError> {
        let available_utxos = self.available_utxos(persistor).await?;
        let Some(fee_utxo) = available_utxos.iter().find(|utxo| {
            utxo.unblinded.asset == *crate::T_L_BTC_ASSET_ID && utxo.unblinded.value > 1000
        }) else {
            return Err(NostradeWalletError::NoUtxo);
        };
        self.wallet
            .send_coins(
                vec![utxo, fee_utxo.outpoint],
                &crate::FEE_ADDRESS,
                420,
                *crate::T_L_BTC_ASSET_ID,
            )
            .await?;
        Ok(())
    }
}

impl PartialEq for NostradeWalletState {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

pub enum NostradeWalletAction {
    Synced,
}

impl Reducible for NostradeWalletState {
    type Action = NostradeWalletAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            NostradeWalletAction::Synced => std::rc::Rc::new(Self {
                synced: self.synced + 1,
                wallet: self.wallet.clone(),
            }),
        }
    }
}

pub type NostradeWalletStore = UseReducerHandle<NostradeWalletState>;

#[function_component(WalletProvider)]
pub fn language_config_provider(props: &yew::html::ChildrenProps) -> HtmlResult {
    let idb_ctx = crate::use_nostrades_db();
    let Some(mut nostr_key) = nostr_minions::use_nostr_key() else {
        return Ok(html! {});
    };
    nostr_key.set_extractable(true);
    let mnemonic = nostr_key
        .mnemonic(nostr_minions::nostro2_signer::Language::English)
        .expect("Failed to get mnemonic");
    nostr_key.set_extractable(false);
    let Ok(wallet) =
        crate::LiquidWebWallet::new(&mnemonic, lwk_wollet::ElementsNetwork::LiquidTestnet)
    else {
        return Ok(html! {});
    };

    let ctx = use_reducer(|| NostradeWalletState { synced: 0, wallet });

    let ctx_clone = ctx.clone();
    yew::suspense::use_future(|| async move { ctx_clone.load(&idb_ctx).await })?;

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
    let address = yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.wallet.address().await {
            Ok(address) => Some(address),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get address: {e}").into());
                None
            }
        }
    })
    .ok()?;
    (*address).clone()
}

#[hook]
pub fn use_wallet_balance() -> Result<
    yew::suspense::UseFutureHandle<std::collections::BTreeMap<elements::AssetId, u64>>,
    yew::suspense::Suspension,
> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.wallet.balance().await {
            Ok(balance) => balance,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get balance: {e}").into());
                std::collections::BTreeMap::new()
            }
        }
    })
}

#[hook]
pub fn use_wallet_utxos()
-> Result<yew::suspense::UseFutureHandle<Vec<lwk_wollet::WalletTxOut>>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let idb_ctx = crate::use_nostrades_db();
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.available_utxos(&idb_ctx).await {
            Ok(utxos) => utxos,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get UTXOs: {e}").into());
                vec![]
            }
        }
    })
}

#[hook]
pub fn use_locked_utxos()
-> Result<yew::suspense::UseFutureHandle<Vec<lwk_wollet::WalletTxOut>>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let idb_ctx = crate::use_nostrades_db();
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.locked_utxos(&idb_ctx).await {
            Ok(utxos) => utxos,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get locked UTXOs: {e}").into());
                vec![]
            }
        }
    })
}

#[hook]
pub fn use_wallet_transactions()
-> Result<yew::suspense::UseFutureHandle<Vec<lwk_wollet::WalletTx>>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.wallet.transactions().await {
            Ok(transactions) => transactions,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get transactions: {e}").into());
                vec![]
            }
        }
    })
}

#[hook]
pub fn use_wallet_locked_utxos()
-> Result<yew::suspense::UseFutureHandle<Vec<lwk_wollet::WalletTxOut>>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let idb_ctx = crate::use_nostrades_db();
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(wallet_ctx.synced, |_| async move {
        match ctx_clone.locked_utxos(&idb_ctx).await {
            Ok(utxos) => utxos,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get locked UTXOs: {e}").into());
                vec![]
            }
        }
    })
}

#[hook]
pub fn use_hard_cancel_swap() -> Callback<elements::OutPoint> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let nostr_key = nostr_minions::use_nostr_key();
    let db_ctx = crate::use_nostrades_db();
    Callback::from(move |utxo: elements::OutPoint| {
        let Some(keypair) = nostr_key.clone() else {
            return;
        };
        let db = db_ctx.clone();
        let relay = relay_ctx.clone();
        let wallet = wallet_ctx.clone();
        yew::platform::spawn_local(async move {
            wallet
                .hard_cancel_swap(utxo, &db)
                .await
                .expect("Failed to cancel swap");
            db.unlock_utxos(&[utxo])
                .await
                .expect("Failed to unlock utxo");
            let mut cancel_note = nostr_minions::nostro2::NostrNote {
                content: "canceled".to_string(),
                kind: 32121,
                ..Default::default()
            };
            cancel_note
                .tags
                .add_parameter_tag(format!("{}:{}", utxo.txid, utxo.vout).as_str());
            keypair
                .sign_note(&mut cancel_note)
                .expect("Failed to sign note");
            let _ = relay.send(cancel_note);
        });
    })
}

#[hook]
pub fn use_soft_cancel_swap() -> Callback<elements::OutPoint> {
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let nostr_key = nostr_minions::use_nostr_key();
    let db_ctx = crate::use_nostrades_db();
    Callback::from(move |outpoint: elements::OutPoint| {
        let Some(keypair) = nostr_key.as_ref() else {
            return;
        };
        let mut cancel_note = nostr_minions::nostro2::NostrNote {
            content: "canceled".to_string(),
            kind: 32121,
            ..Default::default()
        };
        cancel_note
            .tags
            .add_parameter_tag(format!("{}:{}", outpoint.txid, outpoint.vout).as_str());
        keypair
            .sign_note(&mut cancel_note)
            .expect("Failed to sign note");
        let db = db_ctx.clone();
        let relay = relay_ctx.clone();
        yew::platform::spawn_local(async move {
            if db.unlock_utxos([outpoint].as_ref()).await.is_ok() {
                let _ = relay.send(cancel_note);
            }
        });
    })
}

#[hook]
pub fn use_send_coins() -> Callback<(elements::Address, u64, elements::AssetId)> {
    let persistor = crate::use_nostrades_db();
    let wallet = use_context::<NostradeWalletStore>().expect("No wallet context found");
    Callback::from(move |(address, amount, asset_id)| {
        let wallet = wallet.clone();
        let persistor = persistor.clone();
        yew::platform::spawn_local(async move {
            if let Err(e) = wallet
                .normal_coin_send(address, amount, asset_id, &persistor)
                .await
            {
                web_sys::console::error_1(&format!("Failed to send coins: {e}").into());
            }
        });
    })
}
