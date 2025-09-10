use yew::prelude::*;

use crate::ESPLORA_CLIENT;
static MEMPOOL_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

pub async fn is_tx_spent(tx_id: elements::Txid, vout: u32) -> bool {
    let txid = tx_id.to_string();
    web_sys::console::log_1(&format!("Checking tx: {tx_id}").into());
    let txs = MEMPOOL_CLIENT
        .get(format!(
            "https://liquid.network/api/tx/{txid}/outspend/{vout}"
        ))
        .send()
        .await
        .expect("Failed to get tx spend");
    let spent: crate::SpentResponse = txs.json().await.expect("Failed to parse tx spend");
    spent.spent
}

#[function_component(OrderBookScreen)]
pub fn order_book_screen() -> HtmlResult {
    let wallet_ctx = crate::use_wallet_ctx();
    let Some(persistor) = wallet_ctx.persistor().cloned() else {
        return Ok(html! {});
    };
    let offers = yew::suspense::use_future_with((), |_| async move {
        let Ok(orders) = persistor.get_all_offers().await else {
            return vec![];
        };
        let parsed_offers = orders
            .iter()
            .filter_map(|offer| {
                offer
                    .offer
                    .content
                    .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
                    .ok()
            })
            .collect::<Vec<_>>();
       let needed_txs = parsed_offers
           .iter()
           .filter_map(|proposal| proposal.needed_tx().ok())
           .collect::<Vec<_>>();
        let mut valid_txs = vec![];
        for tx in needed_txs {
            if !is_tx_spent(tx, 0).await {
                valid_txs.push(tx);
            }
        }
        web_sys::console::log_1(&format!("Valid TXs: {valid_txs:?}").into());
       //  let Ok(txs) = ESPLORA_CLIENT
       //      .write()
       //      .await
       //      .get_transactions(&needed_txs)
       //      .await
       //  else {
       //      return vec![];
       //  };
        let mut validated_orders = vec![];
        for offer in parsed_offers {
            // let needed_tx = offer.needed_tx().unwrap();
            // let Some(tx) = txs.iter().find(|tx| tx.txid() == needed_tx) else {
            //     continue;
            // };
            let Ok(validated) = offer.insecure_validate() else {
                continue;
            };
            // let Ok(validated) = offer.validate(tx.clone()) else {
            //     continue;
            // };
            validated_orders.push(validated);
        }
        validated_orders
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
