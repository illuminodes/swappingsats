use yew::prelude::*;

static MEMPOOL_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);


#[function_component(OrderBookScreen)]
pub fn order_book_screen() -> HtmlResult {
    let wallet_ctx = crate::use_wallet_ctx();
    let Some(persistor) = wallet_ctx.persistor().cloned() else {
        return Ok(html! {});
    };
    let offers = yew::suspense::use_future_with((), |_| async move {
        let orders = persistor.get_offers_in_last_hour().await;
        let Ok(orders) = orders else {
            return vec![];
        };
        orders
    })?;

    let onclick = Callback::from(move |proposal: lwk_wollet::LiquidexProposal<lwk_wollet::Validated>| {
        let wallet = wallet_ctx.clone();
        yew::platform::spawn_local(async move {
            wallet.liquidex_take(proposal).await.expect("Failed to take");
        });
    });
    Ok(html! {
        <div class="p-4 flex flex-col gap-4 min-h-screen">
            <h1 class="font-semibold">{"Order Book"}</h1>
            {offers.iter().cloned().map(|offer| {
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
                        onclick={onclick.reform(move |_| offer.clone())}
                        class="p-2 border border-gray-200 shadow-lg rounded-xl">
                        {"Swap"}
                    </button>
                    </div>
                }
            }).collect::<Html>()}
        </div>

    })
}
