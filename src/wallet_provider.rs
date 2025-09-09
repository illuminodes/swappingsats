use yew::prelude::*;

static ESPLORA_CLIENT: std::sync::LazyLock<
    tokio::sync::RwLock<lwk_wollet::clients::asyncr::EsploraClient>,
> = std::sync::LazyLock::new(|| {
    lwk_wollet::clients::asyncr::EsploraClientBuilder::new(
        "https://blockstream.info/liquidtestnet/api/",
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
    pub fn loaded(&self) -> bool {
        self.loaded
    }
    pub fn synced(&self) -> bool {
        self.synced > 0
    }
    pub async fn load(&self) -> Result<(), NostradeWalletError> {
        web_sys::console::log_1(&"Loading wallet...".into());
        let mut wollet = self.wollet.write().await;
        let Some(persistor) = &self.persistor else {
            return Ok(());
        };
        let updates = persistor.get_all().await?;

        for update in updates {
            if let Err(e) = wollet.apply_update_no_persist(update.clone()) {
                web_sys::console::error_1(&format!("Failed to apply update: {e}").into());
            }
        }
        web_sys::console::log_1(&"Wallet loaded".into());

        Ok(())
    }
    pub async fn sync(&self) -> Result<(), NostradeWalletError> {
        let mut client = ESPLORA_CLIENT.write().await;
        if client.tip().await.map(|tip| tip.height).ok()
            == Some(self.wollet.read().await.tip().height())
        {
            return Ok(());
        }
        let Some(update) = client.full_scan(&*self.wollet.read().await).await? else {
            return Ok(());
        };
        self.wollet
            .write()
            .await
            .apply_update_no_persist(update.clone())?;
        let Some(persistor) = &self.persistor else {
            return Err(NostradeWalletError::NoPersister);
        };
        persistor.push(update).await?;
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
        let wollet = self.wollet.read().await;
        Ok(wollet.utxos()?)
    }
    pub async fn transactions(&self) -> Result<Vec<lwk_wollet::WalletTx>, NostradeWalletError> {
        let wollet = self.wollet.read().await;
        Ok(wollet.transactions()?)
    }

    pub const FED_PEG_DESC: &str = "wsh(or_d(multi(11,020e0338c96a8870479f2396c373cc7696ba124e8635d41b0ea581112b67817261,02675333a4e4b8fb51d9d4e22fa5a8eaced3fdac8a8cbf9be8c030f75712e6af99,02896807d54bc55c24981f24a453c60ad3e8993d693732288068a23df3d9f50d48,029e51a5ef5db3137051de8323b001749932f2ff0d34c82e96a2c2461de96ae56c,02a4e1a9638d46923272c266631d94d36bdb03a64ee0e14c7518e49d2f29bc4010,031c41fdbcebe17bec8d49816e00ca1b5ac34766b91c9f2ac37d39c63e5e008afb,03079e252e85abffd3c401a69b087e590a9b86f33f574f08129ccbd3521ecf516b,03111cf405b627e22135b3b3733a4a34aa5723fb0f58379a16d32861bf576b0ec2,0318f331b3e5d38156da6633b31929c5b220349859cc9ca3d33fb4e68aa0840174,03230dae6b4ac93480aeab26d000841298e3b8f6157028e47b0897c1e025165de1,035abff4281ff00660f99ab27bb53e6b33689c2cd8dcd364bc3c90ca5aea0d71a6,03bd45cddfacf2083b14310ae4a84e25de61e451637346325222747b157446614c,03cc297026b06c71cbfa52089149157b5ff23de027ac5ab781800a578192d17546,03d3bde5d63bdb3a6379b461be64dad45eabff42f758543a9645afd42f6d424828,03ed1e8d5109c9ed66f7941bc53cc71137baa76d50d274bda8d5e8ffbd6e61fe9a),and_v(v:older(4032),multi(2,03aab896d53a8e7d6433137bbba940f9c521e085dd07e60994579b64a6d992cf79,0291b7d0b1b692f8f524516ed950872e5da10fb1b808b5a526dedc6fed1cf29807,0386aa9372fbab374593466bc5451dc59954e90787f08060964d95c87ef34ca5bb))))#7jwwklk4";
    pub async fn pegin_address(&self) -> Result<String, NostradeWalletError> {
        let wollet = self.wollet.read().await;
        let fed_desc = Self::FED_PEG_DESC.parse().unwrap();
        Ok(wollet
            .pegin_address(None, fed_desc)
            .map(|addr| addr.address().to_string())?)
    }
    pub async fn send_coins(
        &self,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<elements::Txid, NostradeWalletError> {
        let wollet = self.wollet.write().await;
        let mut pset = wollet
            .tx_builder()
            .add_recipient(recipient, amount, asset_id)?
            .fee_rate(Some(200.)) // Adjust fee rate as needed
            .finish()?;
        lwk_common::Signer::sign(&self.signer, &mut pset)?;
        let tx = wollet.finalize(&mut pset)?;
        let tx_id = ESPLORA_CLIENT.write().await.broadcast(&tx).await?;
        let Some(persistor) = &self.persistor else {
            return Err(NostradeWalletError::NoPersister);
        };
        let updates = wollet.updates()?;

        for update in updates {
            if let Err(e) = persistor.push(update.clone()).await {
                web_sys::console::error_1(&format!("Failed to persist update: {e}").into());
            }
        }
        Ok(tx_id)
    }
    pub async fn liquidex_proposal(
        &self,
        utxo: elements::OutPoint,
        recipient: &elements::Address,
        amount: u64,
        asset_id: elements::AssetId,
    ) -> Result<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>, ()> {
        let wollet = self.wollet.read().await;
        let mut pset = wollet
            .tx_builder()
            .liquidex_make(utxo, recipient, amount, asset_id)
            .unwrap()
            .finish()
            .map_err(|e| {
                web_sys::console::error_1(
                    &format!("Failed to build Liquidex proposal: {e}").into(),
                );
            })?;
        lwk_common::Signer::sign(&self.signer, &mut pset).map_err(|_| {
            web_sys::console::error_1(&"Failed to sign Liquidex proposal".into());
        })?;
        lwk_wollet::LiquidexProposal::from_pset(&pset).map_err(|e| {
            web_sys::console::error_1(&format!("Failed to create Liquidex proposal: {e}").into());
        })
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
        return Ok(html! {
            <div class="flex flex-col items-center justify-evenly h-screen w-screen p-4">
                <crate::NostrLogin />
            </div>
        });
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
pub fn use_pegin_address()
-> Result<yew::suspense::UseFutureHandle<Option<String>>, yew::suspense::Suspension> {
    let wallet_ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let ctx_clone = wallet_ctx.clone();
    yew::suspense::use_future_with(
        (wallet_ctx.loaded, wallet_ctx.synced),
        |loaded| async move {
            if !loaded.0 {
                return None;
            }
            match ctx_clone.pegin_address().await {
                Ok(address) => Some(address),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get pegin address: {e}").into());
                    None
                }
            }
        },
    )
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
