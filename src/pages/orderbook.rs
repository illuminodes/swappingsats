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

static MEMPOOL_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

#[function_component(OrderBookScreen)]
pub fn order_book_screen() -> Html {
    html! {
        <div class="p-4 flex flex-col gap-4 min-h-screen">
            <h1 class="font-semibold">{"Order Book"}</h1>
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
pub fn order_book_screen() -> HtmlResult {
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

    let onclick = Callback::from(
        move |val: (String, lwk_wollet::LiquidexProposal<lwk_wollet::Validated>)| {
            let wallet = wallet_ctx.clone();
            let db = db_ctx.clone();
            yew::platform::spawn_local(async move {
                match wallet.liquidex_take(val.1, val.0.clone(), &db).await {
                    Ok(txid) => {
                        // TODO: show user modal with txid and link to explorer
                        // reload ordebook after filtering from persisted swaps
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
    );
    Ok(html! {
            {orders.iter().map(|(note, offer)| {
                let input ={ match offer.input().asset.to_string().as_str() {
                    "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => "L-BTC",
                    "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => "USDT",
                    _ => "Unknown Asset",
                }};
                let input_amount = offer.input().amount;
                let output_asset = match offer.output().asset.to_string().as_str() {
                    "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => "L-BTC",
                    "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => "USDT",
                    _ => "Unknown Asset",
                };
                let output_amount = offer.output().amount;
                html! {
                    <div class="flex flex-row justify-between items-center p-4 border border-gray-200 shadow-lg rounded-xl">
                    <div class="flex flex-col gap-2">
                        <h4 class="font-semibold">{format!("{input} {input_amount}")}</h4>
                        <p class="text-gray-500">{"Swap for"}</p>
                        <h4 class="font-semibold">{format!("{output_asset} {output_amount}")}</h4>
                    </div>
                    <button
                        onclick={
                            let id = note.id.as_ref().unwrap().clone();
                            let offer = offer.clone();
                            onclick.reform(move |_| (id.clone(), offer.clone()))
                        }
                        class="p-2 border border-gray-200 shadow-lg rounded-xl">
                        {"Swap"}
                    </button>
                    </div>
                }
            }).collect::<Html>()}
    })
}
