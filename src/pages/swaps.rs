use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SwapCoinsScreenProps {
    pub utxo_to_swap: UseStateHandle<Option<lwk_wollet::WalletTxOut>>,
}

#[function_component(SwapCoinsScreen)]
pub fn swap_coins_screen() -> HtmlResult {
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
               class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
               <crate::components::ArrowLeft class="size-5" />
           </button>
        }
    } else {
        html! {
           <yew_router::components::Link<crate::router::AppRoute>
               to={crate::router::AppRoute::Home}>
               <button class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
                   <crate::components::ArrowLeft class="size-5" />
               </button>
           </yew_router::components::Link<crate::router::AppRoute>>
        }
    };
    Ok(html!(
        <>
            // Header
            <div class="p-4 flex items-center w-full justify-between">
                {back_option}
                <h1 class="font-semibold">{"Swap"}</h1>
                <button class="w-9"/>
            </div>
            // <SwappableUtxos utxo_to_swap={utxo_to_swap.clone()} />
            {if let Some(_utxo) = (*utxo_to_swap).clone() {
                html! {
                    <UtxoToSwap utxo_to_swap={utxo_to_swap.clone()} />
                }
            } else {
                html! {
                    <Suspense fallback={html! {
                        <div class="w-full items-center justify-center flex p-4">
                            // <crate::components::Loader class="size-8 animate-spin text-gray-500" />
                        </div>
                    }}>
                        <SwappableUtxos utxo_to_swap={utxo_to_swap.clone()} />
                    </Suspense>
                }
            }}
        </>
    ))
}

#[function_component(UtxoToSwap)]
pub fn utxo_to_swap(props: &SwapCoinsScreenProps) -> HtmlResult {
    let wallet_ctx = use_context::<crate::wallet_provider::NostradeWalletStore>()
        .expect("No wallet context found");
    let waiting_for_swap = use_state(|| None::<String>);
    let nostr_key = nostr_minions::use_nostr_key();
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let Some(utxo) = (*props.utxo_to_swap).clone() else {
        return Ok(html! {
            <div class="p-4">
                <p class="text-gray-500">{"No UTXO selected for swap."}</p>
            </div>
        });
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
    let (swap_asset_name, swap_image_url, swap_units, swap_asset_id) = match utxo
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

    let onclick = {
        let utxo_to_swap = props.utxo_to_swap.clone();
        let wallet_ctx = wallet_ctx.clone();
        let waiting_for_swap = waiting_for_swap.clone();
        Callback::from(move |_| {})
    };

    if let Some(req_id) = waiting_for_swap.as_ref() {
        return Ok(html! {
            <SwapNotification req_id={req_id.clone()} />
        });
    }

    Ok(html! {
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
    })
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SwapNotificationProps {
    pub req_id: String,
}

#[function_component(SwapNotification)]
pub fn swap_notification(props: &SwapNotificationProps) -> HtmlResult {

    Ok(html! {
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
    })
}

#[function_component(SwappableUtxos)]
pub fn swappable_utxos(props: &SwapCoinsScreenProps) -> HtmlResult {
    let utxos = crate::wallet_provider::use_wallet_utxos()?;
    let to_swap = props.utxo_to_swap.setter();
    Ok(html! {
            <div>
                <h2 class="font-semibold mb-4 px-6 mt-3">{"Swappable UTXOs"}</h2>
                <div class="max-h-108 overflow-y-auto px-6 snap-y snap-mandatory">
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
                        <div {onclick} class="p-4 mb-2 border border-gray-200 rounded-lg shadow-sm max-w-xs snap-start">
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
