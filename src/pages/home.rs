use yew::prelude::*;

#[function_component(DashboardScreen)]
pub fn dashboard_screen() -> HtmlResult {
    // let quote = crate::quote_provider::use_quote();
    Ok(html! {
        <>
            <div class="bg-white px-4 py-4">
                <div class="flex items-center mb-6">
                    <div class="text-2xl font-bold text-blue-600">{"NOSTRADES WALLET"}</div>
                </div>

                <div class="mb-6">
                    // <p class="text-sm text-gray-600 mb-1">{"Bitcoin Price"}</p>
                    // {if let Some(quote) = quote {
                    //     let date = web_sys::js_sys::Date::new(&web_sys::wasm_bindgen::JsValue::from_f64(
                    //         quote.0 as f64 * 1000.0,
                    //     ));
                    //     let formatted_date = date
                    //         .to_locale_string("en-US", &web_sys::wasm_bindgen::JsValue::UNDEFINED)
                    //         .as_string()
                    //         .unwrap_or("Unknown".to_string());
                    //     html! {
                    //         <>
                    //         <h1 class="text-3xl font-bold">{format!("${:.2}", quote.1.price)}</h1>
                    //         <p class="text-sm text-gray-600">{format!("USD/BTC - {formatted_date}")} </p>
                    //         </>
                    //     }
                    // } else {
                    //     html! {
                    //         <h1 class="text-3xl font-bold">{"Loading..."}</h1>
                    //     }
                    // }}
                </div>

                <MainOptions />

            </div>
            <div class="px-4">

                <h2 class="font-semibold mb-2">{"Spending Accounts"}</h2>

            <Suspense fallback={html! {
                <div class="w-full items-center justify-center flex p-4">
                    <crate::components::LoaderIcon size=8 class="animate-spin text-gray-500" />
                </div>
            }}>
                <AssetList />
            </Suspense>
            </div>
            <div class="px-4">
            <h3 class="font-semibold">{"My Orders"}</h3>
            <Suspense fallback={html! {
                <div class="w-full items-center justify-center flex p-4">
                    <crate::components::LoaderIcon size=8 class="animate-spin text-gray-500" />
                </div>
            }}>
                
                <MyOrders />
            </Suspense>
            </div>

        </>
    })
}

#[function_component(MainOptions)]
fn main_options() -> HtmlResult {
    let navigator = yew_router::hooks::use_navigator().expect("No navigator found");
    Ok(html! {
        <div class="flex shadow-xl border border-gray-200 rounded-2xl ">
            <yew_router::components::Link<crate::router::AppRoute>
                to={crate::router::AppRoute::Receive}>
                <button class="flex justify-center items-center flex-1 gap-2 border-r border-gray-200 p-3 px-6" onclick={Callback::noop()}>
                    <crate::components::ArrowDownLeft class="size-5" />
                    <span class="text-xs">{"Receive"}</span>
                </button>
            </yew_router::components::Link<crate::router::AppRoute>>
            <yew_router::components::Link<crate::router::AppRoute>
                to={crate::router::AppRoute::Swap}>
                <button
                    class="flex justify-center items-center gap-2 p-3 flex-1">
                    <crate::components::ArrowUpDown class="size-5" />
                    <span class="text-xs">{"Swap"}</span>
                </button>
            </yew_router::components::Link<crate::router::AppRoute>>
            <yew_router::components::Link<crate::router::AppRoute>
                to={crate::router::AppRoute::SendCoins}>
                <button class="flex justify-center items-center flex-1 gap-2 border-l border-gray-200 py-3 px-6">
                    <crate::components::ArrowUpRight class="size-5" />
                    <span class="text-xs">{"Send"}</span>
                </button>
            </yew_router::components::Link<crate::router::AppRoute>>
        </div>
    })
}

#[function_component(AssetList)]
fn asset_list() -> HtmlResult {
    let balance = crate::use_wallet_balance()?;
    let liquid_balance = balance
        .iter()
        .find_map(|(k, v)| {
            (Ok(*k) == "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49".parse())
                .then_some(v)
        })
        .copied()
        .unwrap_or(0);

    let usdt_balance = balance
        .iter()
        .find_map(|(k, v)| {
            (Ok(*k) == "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5".parse())
                .then_some(v)
        })
        .copied()
        .unwrap_or(0);

    Ok(html! {
            <>
            // Accounts
                <div class="mb-2">
                    <div class="p-4 shadow-xl rounded-2xl border border-gray-200">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                <img
                                    src="https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png"
                                    alt="Bitcoin" class="size-12 mr-5" />
                                <div>
                                    <h3 class="font-semibold">{"Liquid Bitcoin"}</h3>
                                    <h3 class="text-gray-400">{"L-BTC"}</h3>
                                </div>
                            </div>
                            <div class="text-right">
                                <p class="font-semibold">{liquid_balance as f64 / 100_000_000.0}</p>
                                // <p class="text-sm text-gray-400">{format!("USD {:.2}", liquid_balance as f64 / 100_000_000.0 * quote.1.price  )}</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="mb-4">
                    <div class="p-4 shadow-xl rounded-2xl border border-gray-200">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center">
                                <img
                                    src="https://tether.to/images/logoCircle.png"
                                    alt="Bitcoin" class="size-12 mr-5" />
                                <div>
                                    <h3 class="font-semibold">{"Tether USDt"}</h3>
                                    <h3 class="text-gray-400">{"USD-t"}</h3>
                                </div>
                            </div>
                            <div class="text-right">
                                <p class="font-semibold">{format!("{:.2}", usdt_balance as f64)}</p>
                                <p class="text-sm text-gray-400">{format!("USD {:.2}", usdt_balance as f64)}</p>
                            </div>
                        </div>
                    </div>
                </div>
        </>

    })
}

#[function_component(MyOrders)]
fn my_orders() -> HtmlResult {
    let orders = crate::use_orderbook_ctx();
    let db_ctx = crate::use_nostrades_db();
    let orders = yew::suspense::use_future_with(orders, move |orderbook| async move {
        let offers = orderbook.my_offers().await?;
        let filtered = db_ctx.get_all_swaps().await?;
        let filtered_offers = offers
            .into_iter()
            .filter(|(note, _)| {
                let Some(offer_id) = note.id.as_ref() else {
                    return false;
                };
                !filtered.iter().any(|swap| &swap.id == offer_id)
            })
            .collect::<Vec<_>>();
        Ok::<_, crate::OrderScreenError>(filtered_offers)
    })?;
    let Ok(orders) = orders.as_ref() else {
        return Ok(html! {
            <div class="p-4">
                <p class="text-gray-500">{"Error loading orderbook."}</p>
            </div>
        });
    };
    Ok(html! {
        <div class="p-4 flex flex-col gap-4 min-h-screen">
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
                    </div>
                }
            }).collect::<Html>()}
        </div>
    })
}
