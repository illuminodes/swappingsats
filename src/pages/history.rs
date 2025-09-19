use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct WalletHistoryScreenProps {
    pub transaction: UseStateHandle<Option<lwk_wollet::WalletTx>>,
}
#[function_component(WalletHistoryScreen)]
pub fn swap_coins_screen() -> Html {
    let transaction_detail = use_state(|| None::<lwk_wollet::WalletTx>);
    let _back_option = if transaction_detail.is_some() {
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
        <div class="flex-1 max-w-4xl mx-auto py-5 px-10">
            <h2 class="text-2xl font-bold text-balance mb-5 text-foreground">{"Transactions History"}</h2>
            <div class="overflow-y-auto snap-y snap-mandatory space-y-4">
                { txs.iter().cloned().map(|utxo| {
                    web_sys::console::log_1(&format!("UTXO: {:?}", utxo.type_).into());
                    let net_balances = utxo.balance.clone();
                    let confirmed_msg = utxo.height.map_or_else(|| html! {
                        <span class="text-sm text-muted-foreground">{"Unconfirmed"}</span>
                    }, |height| html! {
                        <span class="text-sm text-muted-foreground">{"Confirmed at block "}{height}</span>
                    });

                    html! {
                        <div class="flex flex-col p-3 border border-gray-200 shadow-lg snap-start rounded-xl bg-card">
                            <div class="flex items-center gap-4">
                                {match utxo.type_.as_str() {
                                    "incoming" => html! {
                                        <div class="p-2 bg-muted rounded-full hidden sm:block">
                                            <crate::components::ArrowDown class="size-4 text-primary" />
                                        </div>
                                    },
                                    "unknown" => html! {
                                        <div class="p-2 bg-chart-3/10 rounded-full hidden sm:block">
                                            <crate::components::ArrowRightLeft class="size-4 text-chart-3" />
                                        </div>
                                    },
                                    _ => html!{
                                        <div class="p-2 bg-destructive/10 rounded-full hidden sm:block">
                                            <crate::components::ArrowUp class="size-4 text-destructive" />
                                        </div>
                                    }
                                }}
                                <div class="flex-1 space-y-2">
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
                                                        "text-sm text-muted-foreground font-light"
                                                    } else {
                                                        "text-foreground text-lg font-semibold"
                                                    }}
                                                >{if utxo.type_ == "incoming" || balance.is_positive() {
                                                    format!("+{balance}")
                                                } else if balance.to_string().starts_with('-') {
                                                        format!("{balance}")
                                                } else {
                                                    format!("-{balance}")
                                                }}</span>
                                            </div>
                                        }
                                    }).collect::<Html>()}
                                </div>
                            </div>
                            <div class="sm:pl-12">
                                {confirmed_msg}
                            </div>
                        </div>
                    }
                }).collect::<Html>()}
            </div>
        </div>
    })
}
