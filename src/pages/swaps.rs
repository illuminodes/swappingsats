use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SwapCoinsScreenProps {
    pub utxo_to_swap: UseStateHandle<Option<lwk_wollet::WalletTxOut>>,
}

#[function_component(SwapTesting)]
pub fn swap_testing(props: &SwapCoinsScreenProps) -> Html {
    let wallet_ctx = crate::use_wallet_ctx();
    let nostr_key = nostr_minions::use_nostr_key();
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let db_ctx = crate::use_nostrades_db();
    let Some(utxo_to_swap) = props.utxo_to_swap.as_ref().cloned() else {
        return html! {
            <div class="p-4">
                <p class="text-gray-500">{"No UTXO selected for swap."}</p>
            </div>
        };
    };

    let onclick = {
        // Extrae solo los campos que necesitas para el closure
        let outpoint = utxo_to_swap.outpoint;
        let asset = utxo_to_swap.unblinded.asset;
        Callback::from(move |form_event: SubmitEvent| {
            form_event.prevent_default();
            let form = form_event.target_unchecked_into::<web_sys::HtmlFormElement>();
            let Some(swap_amount) = form
                .get_with_name("swap_amount")
                .map(wasm_bindgen::JsCast::unchecked_into::<web_sys::HtmlInputElement>)
                .and_then(|input| input.value().parse::<u64>().ok())
            else {
                return;
            };
            let Some(nostr_key) = nostr_key.clone() else {
                return;
            };
            let wallet = wallet_ctx.clone();
            let relay = relay_ctx.clone();
            let db = db_ctx.clone();

            let swap_asset_id = if asset == *crate::T_L_BTC_ASSET_ID {
                *crate::T_USDT_ASSET_ID
            } else if asset == *crate::T_USDT_ASSET_ID {
                *crate::T_L_BTC_ASSET_ID
            } else {
                return;
            };

            yew::platform::spawn_local(async move {
                let proposal = wallet
                    .create_swap_offer(outpoint, swap_amount, swap_asset_id, &db)
                    .await
                    .unwrap();
                let mut proposal_note = nostr_minions::nostro2::NostrNote {
                    content: serde_json::to_string(&proposal).unwrap(),
                    kind: 32121,
                    ..Default::default()
                };
                proposal_note
                    .tags
                    .add_parameter_tag(format!("{}:{}", outpoint.txid, outpoint.vout).as_str());
                nostr_key
                    .sign_note(&mut proposal_note)
                    .expect("Failed to sign note");
                let _ = relay.send(proposal_note);
            });
        })
    };

    html! {
        <form
            class="flex flex-col w-full mx-auto shadow-xl rounded-2xl items-center gap-4 justify-evenly px-6 py-16 bg-card mt-16 border border-foreground/30"
            onsubmit={onclick}
        >
            <div class="max-w-md mx-auto space-y-5">
                <div class="flex items-center justify-center gap-4">
                    <p class="text-muted-foreground text-md text-center">{"Available: "}</p>
                    <p class="text-muted-foreground text-md text-center">{utxo_to_swap.unblinded.value.to_string()}</p>
                </div>
                <input
                    type="number"
                    min="0"
                    step="1"
                    name="swap_amount"
                    placeholder="Swap Amount"
                    class="w-full border border-gray-300 py-2 px-5 rounded-lg"
                />
                <input type="submit" value="Swap Coins" class="bg-primary text-white w-full mx-auto text-center py-3 rounded-lg text-xl hover:cursor-pointer" />
            </div>
        </form>
    }
}

#[function_component(SwapCoinsScreen)]
pub fn swap_coins_screen() -> Html {
    let utxo_to_swap = use_state(|| None::<lwk_wollet::WalletTxOut>);
    let back_option = if utxo_to_swap.is_some() {
        html! {
           <button
                onclick={
                    let utxo_to_swap = utxo_to_swap.clone();
                    Callback::from(move |_| {
                        utxo_to_swap.set(None);
                    })
                }
               class="shadow-lg rounded-xl hover:cursor-pointer flex items-center justify-center">
               <crate::components::ArrowLeft class="size-5" />
           </button>
        }
    } else {
        html! {
           <yew_router::components::Link<crate::router::AppRoute>
               to={crate::router::AppRoute::Home}>
               <button class="shadow-lg rounded-xl hover:cursor-pointer flex items-center justify-center">
                   <crate::components::ArrowLeft class="size-5" />
               </button>
           </yew_router::components::Link<crate::router::AppRoute>>
        }
    };
    html!(
        <div class="flex flex-col items-center h-full max-w-4xl mx-auto flex-1 px-5 md:px-10 py-5">
            // Header
            <div class="flex items-center w-full gap-5">
                { back_option }
                <h2 class="text-2xl font-bold text-balance text-foreground">{"Swappable UXTO's"}</h2>
            </div>
            // <SwappableUtxos utxo_to_swap={utxo_to_swap.clone()} />
            {(*utxo_to_swap).clone().map_or_else(|| html! {
                <Suspense fallback={html! {
                    <div class="w-full items-center justify-center flex p-4">
                        // <crate::components::Loader class="size-8 animate-spin text-gray-500" />
                    </div>
                }}>
                    <SwappableUtxos utxo_to_swap={utxo_to_swap.clone()} />
                    <LockedUtxos utxo_to_swap={utxo_to_swap.clone()} />
                </Suspense>
            }, |_utxo| html! {
                // <UtxoToSwap utxo_to_swap={utxo_to_swap.clone()} />
                <yew::suspense::Suspense fallback={html! {
                    <div class="w-full items-center justify-center flex p-4">
                        // <crate::components::Loader class="size-8 animate-spin text-gray-500" />
                    </div>
                }}>
                    <SwapTesting  utxo_to_swap={utxo_to_swap.clone()} />
                </yew::suspense::Suspense>
            })}
        </div>
    )
}

#[function_component(UtxoToSwap)]
pub fn utxo_to_swap(props: &SwapCoinsScreenProps) -> Html {
    let waiting_for_swap = use_state(|| None::<String>);
    let Some(utxo) = (*props.utxo_to_swap).clone() else {
        return html! {
            <div class="p-4">
                <p class="text-gray-500">{"No UTXO selected for swap."}</p>
            </div>
        };
    };
    let (asset_name, image_url, units) = match utxo.unblinded.asset.to_string().as_str() {
        "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => (
            "L-BTC",
            "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
            "sats",
        ),
        "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => {
            ("USDT", "https://tether.to/images/logoCircle.png", "USD")
        }
        _ => ("Unknown Asset", "", "Unknown"),
    };
    let (swap_asset_name, swap_image_url, _swap_units, _swap_asset_id) = match utxo
        .unblinded
        .asset
        .to_string()
        .as_str()
    {
        "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => (
            "USDT",
            "https://tether.to/images/logoCircle.png",
            "USD",
            "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5",
        ),
        "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => (
            "L-BTC",
            "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
            "sats",
            "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49",
        ),
        _ => ("Unknown Asset", "", "Unknown", ""),
    };

    let onclick = { Callback::from(move |_| {}) };

    if let Some(req_id) = waiting_for_swap.as_ref() {
        return html! {
            <SwapNotification req_id={req_id.clone()} />
        };
    }

    html! {
            <div class="flex flex-col items-center gap-4 justify-evenly h-108 px-6">
                <div class="flex items-center gap-4 w-full justify-evenly border border-gray-200 shadow-lg p-4 rounded-xl">
                    <div class="flex flex-col items-center gap-2">
                        <img src={image_url} alt={asset_name} class="size-12" />
                        <h3 class="font-semibold">{asset_name}</h3>
                    </div>
                    <p class="font-semibold text-xl">{format!("{} {units}", utxo.unblinded.value)}</p>
                </div>
                <h3 class="text-lg font-semibold">{"Swap For"}</h3>
                <div class="flex items-center gap-4 w-full justify-evenly border border-gray-200 shadow-lg p-4 rounded-xl">
                    <div class="flex flex-col items-center gap-2">
                        <img src={swap_image_url} alt={swap_asset_name} class="size-12" />
                        <h3 class="font-semibold">{swap_asset_name}</h3>
                    </div>
                    // <p class="font-semibold text-xl">{format!("{swap_amount:.2} {swap_units}")}</p>
                </div>
                <button
                    {onclick}
                    class="mt-4 bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600">
                    {"Swap Now"}
                </button>
            </div>
    }
}

#[function_component(LockedUtxos)]
pub fn locked_utxos(props: &SwapCoinsScreenProps) -> HtmlResult {
    let locked_utxos = crate::use_wallet_locked_utxos()?;
    let to_swap = props.utxo_to_swap.setter();
    let hard_cancel_swap = crate::use_hard_cancel_swap();
    let soft_cancel_swap = crate::use_soft_cancel_swap();
    Ok(html! {
            <div class="w-full">
                <h2 class="font-semibold mb-4 px-6 mt-3">{"Locked UTXOs"}</h2>
                <div class="max-h-108 overflow-y-auto px-6 snap-y snap-mandatory">
                    { locked_utxos.iter().map(|utxo| {
                        let (image_url, asset_name, units) = match utxo.unblinded.asset.to_string().as_str() {
                            "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => (
                                "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
                                "L-BTC",
                                "sats",
                            ),
                            "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => (
                                "https://tether.to/images/logoCircle.png",
                                "USDT",
                                "USD",
                            ),
                            _ => ("", "Unknown Asset", "Unknown"),
                        };
                        let onclick = {
                            let utxo_to_swap = to_swap.clone();
                            let utxo = utxo.clone();
                            Callback::from(move |_| {
                                utxo_to_swap.set(Some(utxo.clone()));
                            })
                        };
                        let tx_id = utxo.outpoint;
                        let soft_cancel_swap = soft_cancel_swap.clone().reform(move |_| tx_id);
                        let hard_cancel_swap = hard_cancel_swap.clone().reform(move |_| tx_id);

                        html! {
                            <div {onclick} class="p-4 mb-2 border border-gray-200 rounded-lg shadow-sm max-w-xs snap-start">
                                <div class="flex items-center">
                                    <img src={image_url} alt={asset_name} class="size-12 mr-4" />
                                    <div>
                                        <h3 class="font-semibold">{asset_name}</h3>
                                        <p class="text-gray-400">{format!("{} {units}", utxo.unblinded.value)}</p>
                                        <button
                                            onclick={soft_cancel_swap}
                                            class="mt-4 bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600">
                                            {"Soft Cancel"}
                                        </button>
                                        <button
                                            onclick={hard_cancel_swap}
                                            class="mt-4 bg-red-500 text-white px-4 py-2 rounded-lg hover:bg-red-600">
                                            {"Hard Cancel"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct SwapNotificationProps {
    pub req_id: String,
}

#[function_component(SwapNotification)]
pub fn swap_notification(props: &SwapNotificationProps) -> Html {
    html! {
        <div class="p-4 border border-gray-200 rounded-lg shadow-sm max-w-xs">
            <h3 class="font-semibold mb-2">{"Swap Accepted"}</h3>
            <p class="text-gray-500 text-wrap max-w-xs break-all">{format!("Swap Tx ID: {}", props.req_id)}</p>
            <a href={format!("https://blockstream.info/liquidtestnet/tx/{}", props.req_id)}
               target="_blank"
               class="text-blue-500 hover:underline mt-2 block">
               // <crate::components::ExternalLink class="size-4 inline mr-1" />
               {"View Transaction"}
            </a>
        </div>
    }
}

#[function_component(SwappableUtxos)]
pub fn swappable_utxos(props: &SwapCoinsScreenProps) -> HtmlResult {
    let utxos = crate::use_wallet_utxos()?;
    let to_swap = props.utxo_to_swap.setter();
    Ok(html! {
            <div class="flex-1 w-full mt-16">
                <div class="max-h-108 overflow-y-auto px-6 snap-y snap-mandatory w-full">
                    { utxos.iter().map(|utxo| {
                        let (image_url, asset_name, units) = match utxo.unblinded.asset.to_string().as_str() {
                            "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => (
                                "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
                                "L-BTC",
                                "sats"
                            ),
                            "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => (
                                "https://tether.to/images/logoCircle.png",
                                "USDT",
                                "USD"
                            ),
                            _ => ("", "Unknown Asset", "Unknown"),
                        };
                        let onclick = {
                            let utxo_to_swap = to_swap.clone();
                            let utxo = utxo.clone();
                            Callback::from(move |_| {
                                utxo_to_swap.set(Some(utxo.clone()));
                            })
                        };

                        html! {
                            <div {onclick} class="p-4 mb-2 bg-muted border border-primary/30 rounded-lg shadow-sm snap-start w-full hover:cursor-pointer">
                                <div class="flex items-center">
                                    <img src={image_url} alt={asset_name} class="size-12 mr-4" />
                                    <div>
                                        <h3 class="font-semibold">{asset_name}</h3>
                                        <p class="text-gray-400">{format!("{} {units}", utxo.unblinded.value)}</p>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()}
                </div>
            </div>
    })
}
