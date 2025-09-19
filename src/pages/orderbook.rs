use yew::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum OrderScreenError {
    #[error("Orderbook error: {0}")]
    Orderbook(#[from] crate::contexts::OrderbookError),
    #[error("Wallet error: {0}")]
    Wallet(#[from] crate::contexts::NostradeWalletError),
    #[error("Persist error: {0}")]
    Persist(#[from] crate::PersistError),
}

const L_BTC_ASSET_ID: &str = "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49";
const USDT_ASSET_ID: &str = "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5";

#[function_component(OrderBookScreen)]
pub fn order_book_screen() -> Html {
    html! {
        <div class="flex-1 max-w-7xl mx-auto py-5 px-10">
            <h1 class="text-2xl font-bold text-balance text-foreground">{"Order Book"}</h1>
            <yew::suspense::Suspense fallback={html! {
                <div class="w-full items-center justify-center flex p-4">
                    <crate::components::LoaderIcon class="size-8 animate-spin text-gray-500" />
                </div>
            }} >
                <OrderBook />
            </yew::suspense::Suspense>
        </div>
    }
}

#[function_component(OrderBook)]
pub fn order_book() -> HtmlResult {
    let wallet_ctx = crate::use_wallet_ctx();
    let orderbook_ctx = crate::use_orderbook_ctx();
    let db_ctx = crate::use_nostrades_db();
    let db_clone = db_ctx.clone();

    let orders = yew::suspense::use_future_with(orderbook_ctx, |orderbook| async move {
        let offers = orderbook.parsed_offers().await?;
        let filtered = db_clone.get_all_swaps().await?;
        let filtered_offers = offers
            .into_iter()
            .filter(|(note, _)| {
                let Some(offer_id) = note.id.as_ref() else {
                    return false;
                };
                !filtered.iter().any(|swap| &swap.id == offer_id)
            })
            .collect::<Vec<_>>();
        Ok::<_, OrderScreenError>(filtered_offers)
    })?;

    let Ok(orders) = orders.as_ref() else {
        return Ok(html! {
            <div class="p-4">
                <p class="text-gray-500">{"Error loading orderbook."}</p>
            </div>
        });
    };

    let (buy_orders, sell_orders): (Vec<_>, Vec<_>) =
        orders.iter().cloned().partition(|(_, offer)| {
            offer.input().asset.to_string() == L_BTC_ASSET_ID
                && offer.output().asset.to_string() == USDT_ASSET_ID
        });

    let onclick = create_swap_handler(wallet_ctx, db_ctx);

    Ok(html! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-10">
            <div class="bg-white rounded-lg shadow-lg p-6">
                <h2 class="text-xl font-semibold text-primary mb-4 flex items-center">
                    <span class="bg-muted text-primary text-xs font-medium px-2.5 py-0.5 rounded mr-2">
                        {"BUY"}
                    </span>
                    {"L-BTC"}
                </h2>
                {render_orders_section(&buy_orders, &onclick, true)}
            </div>

            <div class="bg-white rounded-lg shadow-lg p-6">
                <h2 class="text-xl font-semibold text-red-600 mb-4 flex items-center">
                    <span class="bg-red-100 text-red-800 text-xs font-medium px-2.5 py-0.5 rounded mr-2">
                        {"SELL"}
                    </span>
                    {"L-BTC"}
                </h2>
                {render_orders_section(&sell_orders, &onclick, false)}
            </div>
        </div>
    })
}

fn create_swap_handler(
    wallet_ctx: crate::contexts::NostradeWalletStore,
    db_ctx: crate::contexts::NostradesIdb,
) -> Callback<(String, lwk_wollet::LiquidexProposal<lwk_wollet::Validated>)> {
    Callback::from(
        move |val: (String, lwk_wollet::LiquidexProposal<lwk_wollet::Validated>)| {
            let wallet = wallet_ctx.clone();
            let db = db_ctx.clone();
            yew::platform::spawn_local(async move {
                match wallet.liquidex_take(val.1, val.0.clone(), &db).await {
                    Ok(_txid) => {
                        web_sys::console::log_1(&"Swap Taken".into());
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to take swap: {e}").into());
                        if let Err(e) = db
                            .push_swap(crate::PersistedSwap::new(
                                val.0.clone(),
                                crate::SwapStatus::Failed,
                            ))
                            .await
                        {
                            web_sys::console::error_1(
                                &format!("Failed to persist swap: {e}").into(),
                            );
                        }
                    }
                }
            });
        },
    )
}

fn render_orders_section(
    orders: &[(
        nostr_minions::nostro2::NostrNote,
        lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
    )],
    onclick: &Callback<(String, lwk_wollet::LiquidexProposal<lwk_wollet::Validated>)>,
    is_buy_section: bool,
) -> Html {
    if orders.is_empty() {
        return html! {
            <div class="text-center py-8">
                <p class="text-gray-500">{"No orders available"}</p>
            </div>
        };
    }

    html! {
        <div class="space-y-3">
            {orders.iter().map(|(note, offer)| {
                render_order_card(note, offer, onclick, is_buy_section)
            }).collect::<Html>()}
        </div>
    }
}

#[allow(clippy::similar_names)]
fn render_order_card(
    note: &nostr_minions::nostro2::NostrNote,
    offer: &lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
    onclick: &Callback<(String, lwk_wollet::LiquidexProposal<lwk_wollet::Validated>)>,
    is_buy_section: bool,
) -> Html {
    let (selling_asset, selling_amount) =
        get_asset_info(&offer.input().asset, offer.input().amount);
    let (wanting_asset, wanting_amount) =
        get_asset_info(&offer.output().asset, offer.output().amount);

    let price_per_unit = calculate_price_per_unit(offer, is_buy_section);
    let timestamp = note.created_at.try_into().map_or_else(
        |_| {
            web_sys::console::warn_1(&format!("Invalid timestamp: {}", note.created_at).into());
            "Invalid time".to_string()
        },
        format_timestamp_safe,
    );

    let pubkey = format_pubkey(&note.pubkey);
    let order_id = note.id.clone().unwrap_or_else(|| "unknown".to_string());

    html! {
        <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow overflow-clip">
            <div class="flex flex-col justify-between items-center gap-y-5">
                <div class="flex-1 w-full">
                    <div class="flex flex-col sm:flex-row lg:flex-col xl:flex-row space-x-6 space-y-3 mb-3">
                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase">{"Price"}</span>
                            <span class="text-lg font-semibold text-gray-900 whitespace-nowrap">
                                {format!("${:.2}", price_per_unit)}
                            </span>
                        </div>

                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase">{"Quantity"}</span>
                            <span class="text-lg font-semibold text-gray-900 whitespace-nowrap">
                                {format!("{} {}", format_amount(selling_amount), selling_asset)}
                            </span>
                        </div>

                        <div class="flex flex-col">
                            <span class="text-xs text-gray-500 uppercase">{"Total"}</span>
                            <span class="text-lg font-semibold text-gray-900 whitespace-nowrap">
                                {format!("{} {}", format_amount(wanting_amount), wanting_asset)}
                            </span>
                        </div>
                    </div>

                    <div class="flex flex-col 2xl:flex-row gap-2 text-xs text-foreground w-full">
                        <div class="whitespace-nowrap">
                            <span class="font-medium">{"ID: "}</span>
                            <span>{
                                if order_id.len() >= 16 {
                                    format!("{}...{}", &order_id[..8], &order_id[order_id.len()-8..])
                                } else {
                                    order_id.clone()
                                }
                            }</span>
                        </div>
                        <div class="whitespace-nowrap">
                            <span class="font-medium">{"Time: "}</span>
                            <span>{timestamp}</span>
                        </div>
                        <div class="whitespace-nowrap">
                            <span class="font-medium">{"Pubkey: "}</span>
                            <span>{pubkey}</span>
                        </div>
                    </div>
                </div>

                <div class="w-full flex-shrink-0">
                    <button
                        onclick={
                            let id = note.id.clone().unwrap_or_else(|| "unknown".to_string());
                            let offer = offer.clone();
                            onclick.reform(move |_| (id.clone(), offer.clone()))
                        }
                        class={format!(
                            "w-full py-3 rounded-lg font-medium hover:cursor-pointer {}",
                            if is_buy_section {
                                "bg-primary text-white"
                            } else {
                                "bg-red-600 text-white"
                            }
                        )}>
                        {if is_buy_section { "Buy" } else { "Sell" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

fn get_asset_info(asset: &elements::AssetId, amount: u64) -> (&'static str, u64) {
    let asset_str = asset.to_string();
    let asset_name = match asset_str.as_str() {
        L_BTC_ASSET_ID => "L-BTC",
        USDT_ASSET_ID => "USDT",
        _ => "Unknown",
    };
    (asset_name, amount)
}

#[allow(clippy::cast_precision_loss)]
fn calculate_price_per_unit(
    offer: &lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
    is_buy_section: bool,
) -> f64 {
    let input_amount = offer.input().amount as f64;
    let output_amount = offer.output().amount as f64;

    if is_buy_section {
        let btc_amount = input_amount / 100_000_000.0;
        if btc_amount > 0.0 {
            output_amount / btc_amount
        } else {
            0.0
        }
    } else {
        let btc_amount = output_amount / 100_000_000.0;
        if btc_amount > 0.0 {
            input_amount / btc_amount
        } else {
            0.0
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_amount(amount: u64) -> String {
    if amount >= 100_000_000 {
        let btc = amount as f64 / 100_000_000.0;
        format!("{btc:.8}")
    } else if amount > 1000 {
        format!("{:.2}", amount as f64)
    } else {
        format!("{amount}")
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_timestamp_safe(timestamp: u64) -> String {
    let timestamp_f64 = timestamp as f64;

    if timestamp_f64 > (js_sys::Date::now() + 31_536_000_000.0) {
        return "Future".to_string();
    }

    if timestamp_f64 < 946_684_800.0 {
        return "Past".to_string();
    }

    let timestamp_ms = if timestamp < 1_000_000_000_000 {
        timestamp_f64 * 1000.0
    } else {
        timestamp_f64
    };

    wasm_bindgen::JsValue::from_f64(timestamp_ms)
        .as_f64()
        .map_or_else(
            || {
                web_sys::console::warn_1(&format!("Invalid timestamp: {timestamp}").into());
                "Unknown time".to_string()
            },
            |safe_ms| {
                let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(safe_ms));
                date.to_locale_string("en-US", &js_sys::Object::new())
                    .as_string()
                    .map_or_else(|| "Invalid date".to_string(), |formatted| formatted)
            },
        )
}

fn format_pubkey(pubkey: &str) -> String {
    if pubkey.len() >= 16 {
        format!("{}...{}", &pubkey[..8], &pubkey[pubkey.len() - 8..])
    } else {
        pubkey.to_string()
    }
}
