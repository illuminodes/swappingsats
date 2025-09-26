use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct SwapCoinsScreenProps {
    pub utxo_to_swap: UseStateHandle<Option<lwk_wollet::WalletTxOut>>,
}

const STATIC_BTC_USD_PRICE: f64 = 65000.0;

#[function_component(SwapTesting)]
pub fn swap_testing(props: &SwapCoinsScreenProps) -> Html {
    let wallet_ctx = crate::use_wallet_ctx();
    let nostr_key = nostr_minions::use_nostr_key();
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let db_ctx = crate::use_nostrades_db();
    let requested_amount = use_state(String::new);
    let navigator = yew_router::hooks::use_navigator().unwrap();

    let Some(utxo_to_swap) = props.utxo_to_swap.as_ref().cloned() else {
        return html! {
            <div class="p-4">
                <p class="text-gray-500">{"No UTXO selected for swap."}</p>
            </div>
        };
    };

    let (offering_asset_name, offering_image_url, offering_units) =
        get_asset_info(&utxo_to_swap.unblinded.asset);
    let swap_asset_id = get_swap_asset_id(&utxo_to_swap.unblinded.asset);
    let (requesting_asset_name, requesting_image_url, requesting_units) =
        get_asset_info(&swap_asset_id);

    let offering_amount = utxo_to_swap.unblinded.value;

    let requested_value = requested_amount.parse::<u64>().unwrap_or(0);

    let market_suggestion = calculate_market_rate(&utxo_to_swap.unblinded.asset, offering_amount);

    let price_info = if requested_value > 0 {
        calculate_price_comparison(
            &utxo_to_swap.unblinded.asset,
            offering_amount,
            requested_value,
        )
    } else {
        PriceComparison::empty()
    };

    let on_requested_change = {
        let requested_amount = requested_amount.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            requested_amount.set(input.value());
        })
    };

    let onclick = {
        let outpoint = utxo_to_swap.outpoint;

        Callback::from(move |form_event: SubmitEvent| {
            form_event.prevent_default();

            if requested_value == 0 {
                web_sys::console::error_1(&"Must specify amount to request".into());
                return;
            }

            let Some(nostr_key) = nostr_key.clone() else {
                web_sys::console::error_1(&"No nostr key found".into());
                return;
            };

            let wallet = wallet_ctx.clone();
            let relay = relay_ctx.clone();
            let db = db_ctx.clone();
            let navigator = navigator.clone(); // Clonar para el async block

            web_sys::console::log_1(&format!(
                "Creating swap offer: Offering entire UTXO ({offering_amount} {offering_units}) for {requested_value} {requesting_units}",
            ).into());

            yew::platform::spawn_local(async move {
                match wallet
                    .create_swap_offer(outpoint, requested_value, swap_asset_id, &db)
                    .await
                {
                    Ok(proposal) => {
                        let mut proposal_note = nostr_minions::nostro2::NostrNote {
                            content: serde_json::to_string(&proposal).unwrap(),
                            kind: 32121,
                            ..Default::default()
                        };
                        proposal_note.tags.add_parameter_tag(
                            format!("{}:{}", outpoint.txid, outpoint.vout).as_str(),
                        );

                        if let Err(e) = nostr_key.sign_note(&mut proposal_note) {
                            web_sys::console::error_1(&format!("Failed to sign note: {e}").into());
                            return;
                        }

                        let _ = relay.send(proposal_note);
                        web_sys::console::log_1(&"Swap offer created successfully".into());

                        navigator.push(&crate::router::AppRoute::Home);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to create swap offer: {e}").into(),
                        );
                    }
                }
            });
        })
    };

    let is_valid_swap = requested_value > 0;

    html! {
        <div class="w-full max-w-lg mx-auto mt-8">
            <div class="bg-white rounded-2xl shadow-xl border border-gray-200 p-6">
                <div class="text-center mb-6">
                    <h2 class="text-xl font-bold text-gray-900 mb-2">{"Create Swap Offer"}</h2>
                    <p class="text-sm text-gray-600">{"Set your price for this complete UTXO"}</p>
                </div>

                <form onsubmit={onclick} class="space-y-6">
                    <div class="bg-blue-50 rounded-xl p-4 border-1 border-blue-200">
                        <label class="block text-sm font-medium text-foreground mb-3">
                            {"You're offering"}
                        </label>
                        <div class="flex items-center justify-between">
                            <div class="flex items-center space-x-3">
                                <img src={offering_image_url} alt={offering_asset_name} class="size-10 rounded-full" />
                                <div>
                                    <div class="font-semibold text-gray-900">{offering_asset_name}</div>
                                    <div class="text-sm text-muted-foreground">{"Complete UTXO amount"}</div>
                                </div>
                            </div>
                            <div class="text-right">
                                <div class="text-lg text-gray-900 bg-gray-100 px-3 py-2 rounded-md">
                                    {format_display_amount(offering_amount, &utxo_to_swap.unblinded.asset)}
                                </div>
                                <div class="text-sm text-muted-foreground mt-1">{offering_units}</div>
                            </div>
                        </div>
                    </div>

                    <div class="flex justify-center">
                        <div class="bg-blue-100 p-2 rounded-full">
                            <svg class="size-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"></path>
                            </svg>
                        </div>
                    </div>

                    <div class="bg-muted rounded-xl p-4 border-1 border-primary/30">
                        <label class="block text-sm font-medium text-foreground">
                            {"You're want in return"}
                        </label>
                        <div class="flex items-center justify-between">
                            <div class="flex items-center space-x-3">
                                <img src={requesting_image_url} alt={requesting_asset_name} class="size-10 rounded-full" />
                                <div>
                                    <div class="font-semibold text-gray-900">{requesting_asset_name}</div>
                                    <div class="text-sm text-gray-500">{"Amount you want"}</div>
                                </div>
                            </div>
                            <div class="text-right">
                                <input
                                    type="number"
                                    name="requested_amount"
                                    placeholder="0"
                                    min="1"
                                    value={(*requested_amount).clone()}
                                    onchange={on_requested_change}
                                    class="text-right text-lg bg-white border border-gray-300 rounded-md px-2 py-2 outline-none w-40 text-gray-900 placeholder-gray-400"
                                />
                                <div class="text-sm text-gray-500 mt-1">{requesting_units}</div>
                                <div class="text-xs text-gray-400 mt-1">
                                    {format!("Market: ~{}", format_display_amount(market_suggestion, &swap_asset_id))}
                                </div>
                            </div>
                        </div>
                    </div>

                    {if requested_value > 0 {
                        html! {
                            <div class="bg-yellow-50 rounded-xl p-4 border border-yellow-200">
                                <div class="flex items-start space-x-3">
                                    <div class="flex-shrink-0">
                                        <svg class="size-5 text-yellow-600 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                        </svg>
                                    </div>
                                    <div class="flex-1">
                                        <h4 class="text-sm font-medium text-yellow-800 mb-2">{"Your Pricing"}</h4>
                                        <div class="space-y-1 text-sm text-yellow-700">
                                            <div class="flex justify-between">
                                                <span>{"Your Rate:"}</span>
                                                <span class="font-medium">{price_info.rate_display}</span>
                                            </div>
                                            <div class="flex justify-between">
                                                <span>{"Market Rate:"}</span>
                                                <span class="font-medium">{format!("${:.0}/BTC", STATIC_BTC_USD_PRICE)}</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    <button
                        type="submit"
                        disabled={!is_valid_swap}
                        class={format!(
                            "w-full py-3 px-4 rounded-xl text-md transition-all duration-200 {}",
                            if is_valid_swap {
                                "bg-primary text-white cursor-pointer"
                            } else {
                                "bg-gray-200 text-foreground cursor-not-allowed"
                            }
                        )}
                    >
                        {if requested_value == 0 {
                            "Enter amount you want in return"
                        } else {
                            "Create Swap Offer"
                        }}
                    </button>

                    <div class="text-xs text-foreground text-center space-y-1">
                        <div>{"You're offering the complete UTXO shown above."}</div>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[derive(Debug, Clone)]
struct PriceComparison {
    rate_display: String,
}

impl PriceComparison {
    fn empty() -> Self {
        Self {
            rate_display: "Enter amount".to_string(),
        }
    }
}

fn get_asset_info(asset_id: &elements::AssetId) -> (&'static str, &'static str, &'static str) {
    match asset_id.to_string().as_str() {
        "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => (
            "L-BTC",
            "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
            "sats",
        ),
        "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => {
            ("USDT", "https://tether.to/images/logoCircle.png", "USDT")
        }
        _ => ("Unknown", "", "Unknown"),
    }
}

fn get_swap_asset_id(current_asset: &elements::AssetId) -> elements::AssetId {
    if *current_asset == *crate::T_L_BTC_ASSET_ID {
        *crate::T_USDT_ASSET_ID
    } else {
        *crate::T_L_BTC_ASSET_ID
    }
}

#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn calculate_market_rate(offering_asset: &elements::AssetId, offering_amount: u64) -> u64 {
    if *offering_asset == *crate::T_L_BTC_ASSET_ID {
        let btc_amount = offering_amount as f64 / 100_000_000.0;
        let usdt_amount = btc_amount * STATIC_BTC_USD_PRICE;
        usdt_amount.round() as u64
    } else if *offering_asset == *crate::T_USDT_ASSET_ID {
        let btc_amount = offering_amount as f64 / STATIC_BTC_USD_PRICE;
        let satoshi_amount = btc_amount * 100_000_000.0;
        satoshi_amount.round() as u64
    } else {
        0
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_display_amount(amount: u64, asset_id: &elements::AssetId) -> String {
    if *asset_id == *crate::T_L_BTC_ASSET_ID {
        let btc = amount as f64 / 100_000_000.0;
        if btc >= 1.0 {
            format!("{btc:.4}")
        } else if btc >= 0.001 {
            format!("{btc:.6}")
        } else {
            format!("{btc:.8}")
        }
    } else {
        format!("{amount}")
    }
}

#[allow(clippy::cast_precision_loss)]
fn calculate_price_comparison(
    offering_asset: &elements::AssetId,
    offering_amount: u64,
    requested_amount: u64,
) -> PriceComparison {
    if offering_amount == 0 || requested_amount == 0 {
        return PriceComparison::empty();
    }

    if *offering_asset == *crate::T_L_BTC_ASSET_ID {
        let btc_amount = offering_amount as f64 / 100_000_000.0;
        let rate_per_btc = requested_amount as f64 / btc_amount;
        let rate_display = format!("1 L-BTC = {rate_per_btc:.0} USDT");

        PriceComparison { rate_display }
    } else if *offering_asset == *crate::T_USDT_ASSET_ID {
        let btc_amount = requested_amount as f64 / 100_000_000.0;
        let rate_per_btc = offering_amount as f64 / btc_amount;
        let rate_display = format!("1 L-BTC = {rate_per_btc:.0} USDT");

        PriceComparison { rate_display }
    } else {
        PriceComparison::empty()
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
            <div class="flex items-center w-full gap-5">
                { back_option }
                <h2 class="text-2xl font-bold text-balance text-foreground">{"Swappable UXTO's"}</h2>
            </div>
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
