use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct WalletHistoryScreenProps {
    pub transaction: UseStateHandle<Option<lwk_wollet::WalletTx>>,
}
#[function_component(WalletHistoryScreen)]
pub fn swap_coins_screen() -> Html {
    let transaction_detail = use_state(|| None::<lwk_wollet::WalletTx>);
    let back_option = if transaction_detail.is_some() {
        html! {
           <button
                onclick={
                    let utxo_to_swap = transaction_detail.clone();
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
    html!(
        <>
            <div class="p-4 flex items-center w-full justify-between">
                {back_option}
                <h1 class="font-semibold">{"History"}</h1>
                <button class="w-9"/>
            </div>
            // <SwappableUtxos utxo_to_swap={utxo_to_swap.clone()} />
            {
                (*transaction_detail).clone().map_or_else(
                    || html! { <TxHistory transaction={transaction_detail} /> },
                    |_utxo| html! {}
                )
            }
        </>
    )
}

#[function_component(TxHistory)]
pub fn tx_history(_props: &WalletHistoryScreenProps) -> HtmlResult {
    let txs = crate::use_wallet_transactions()?;
    // let tx_handle = props.transaction.clone();
    Ok(html! {
            <div>
                <h2 class="font-semibold mb-4 px-6 mt-3">{"Transactions"}</h2>
                <div class="max-h-108 overflow-y-auto px-6 snap-y snap-mandatory space-y-4">
                { txs.iter().cloned().map(|utxo| {
                    let net_balances = utxo.balance.clone();
                    let confirmed_msg = utxo.height.map_or_else(
                        || html! {
                            <span class="text-xs text-gray-500">{"Unconfirmed"}</span>
                        },
                        |height| html! {
                            <span class="text-xs text-gray-500">{"Confirmed at block "}{height}</span>
                        }
                    );
                    html! {
                        <div class="flex flex-col p-3 border border-gray-200 shadow-lg snap-start rounded-xl">
                            {net_balances.iter().map(|(asset_id, balance)| {
                                let asset_name = match asset_id.to_string().as_str() {
                                    "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49" => "Liquid Bitcoin",
                                    "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5" => "Tether USD",
                                    _ => "Unknown Asset",
                                };
                                html! {
                                    <div class="flex justify-between items-center">
                                        <span class="font-semibold">{asset_name}</span>
                                        <span
                                            class={if balance.is_negative() {
                                                "text-red-500"
                                            } else {
                                                "text-green-500"
                                            }}
                                            >{balance}</span>
                                    </div>
                                }
                            }).collect::<Html>()}
                            {confirmed_msg}
                        </div>
                    }
                }).collect::<Html>()}
                </div>
            </div>
    })
}
