use std::str::FromStr;

use crate::components::{ArrowLeft, show_error_toast, show_success_toast};
use shady_minions::ui::Modal;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedAsset {
    LiquidBitcoin,
    Tether,
}

impl SupportedAsset {
    const fn get_asset_info(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::LiquidBitcoin => (
                "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
                "Liquid Bitcoin",
                "L-BTC",
            ),
            Self::Tether => ("https://tether.to/images/logoCircle.png", "Tether", "USDT"),
        }
    }

    fn get_asset_id(&self) -> elements::AssetId {
        match self {
            Self::LiquidBitcoin => *crate::T_L_BTC_ASSET_ID,
            Self::Tether => *crate::T_USDT_ASSET_ID,
        }
    }
}

#[function_component(SendCoinsScreen)]
pub fn send_coins_screen() -> Html {
    html!(
        <div class="flex flex-col items-center max-w-4xl mx-auto flex-1 px-5 md:px-10 py-5">
            <div class="flex items-center w-full gap-5">
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::Home}>
                    <button class="size-5 flex items-center justify-center hover:cursor-pointer">
                       <ArrowLeft class="size-5" />
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
                <h2 class="text-2xl font-bold text-balance text-foreground">{"Send"}</h2>
            </div>

            <SendCoinForm />
        </div>
    )
}

#[allow(clippy::redundant_clone)]
#[function_component(SendCoinForm)]
fn send_coin_form() -> Html {
    let selected_asset = use_state(|| SupportedAsset::LiquidBitcoin);
    let address_valid = use_state(|| None::<bool>);
    let show_success_modal = use_state(|| false);
    let transaction_id = use_state(String::new);
    let recipient_address = use_state(String::new);
    let asset_amount = use_state(String::new);
    let navigator = use_navigator().unwrap();

    let wallet_ctx = crate::use_wallet_ctx();
    let nostrades_db = crate::use_nostrades_db();

    let (asset_img, asset_name, asset_symbol) = selected_asset.get_asset_info();

    let send_coins_with_modal = {
        let show_success_modal = show_success_modal.clone();
        let transaction_id = transaction_id.clone();
        let wallet_ctx = wallet_ctx.clone();
        let nostrades_db = nostrades_db.clone();
        let recipient_address = recipient_address.clone();
        let asset_amount = asset_amount.clone();
        let address_valid = address_valid.clone();

        Callback::from(
            move |(address, amount, asset_id): (elements::Address, u64, elements::AssetId)| {
                let wallet = wallet_ctx.clone();
                let persistor = nostrades_db.clone();
                let show_modal = show_success_modal.clone();
                let tx_id = transaction_id.clone();
                let recipient_addr = recipient_address.clone();
                let asset_amt = asset_amount.clone();
                let addr_valid = address_valid.clone();

                yew::platform::spawn_local(async move {
                    match wallet
                        .normal_coin_send(address, amount, asset_id, &persistor)
                        .await
                    {
                        Ok(txid) => {
                            web_sys::console::log_1(
                                &format!("Transaction successful: {txid}").into(),
                            );
                            tx_id.set(txid.to_string());
                            show_modal.set(true);

                            show_success_toast("Transaction sent successfully!");

                            recipient_addr.set(String::new());
                            asset_amt.set(String::new());
                            addr_valid.set(None);
                        }
                        Err(e) => {
                            web_sys::console::error_1(&format!("Failed to send coins: {e}").into());

                            let error_message = if e.to_string().contains("insufficient") {
                                "Insufficient funds for this transaction"
                            } else if e.to_string().contains("fee") {
                                "Unable to calculate network fees"
                            } else if e.to_string().contains("network") {
                                "Network error - please try again"
                            } else {
                                "Failed to send transaction"
                            };

                            show_error_toast(error_message);
                        }
                    }
                });
            },
        )
    };

    let onsubmit = {
        let selected_asset = selected_asset.clone();
        let address_valid = address_valid.clone();
        let recipient_address = recipient_address.clone();
        let asset_amount = asset_amount.clone();
        let send_coins_with_modal = send_coins_with_modal;

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if address_valid.is_none() || address_valid.is_some_and(|valid| !valid) {
                show_error_toast("Please enter a valid recipient address");
                return;
            }

            let recipient_address_value = (*recipient_address).clone();
            let Ok(elements_address) = elements::Address::from_str(&recipient_address_value) else {
                show_error_toast("Invalid recipient address format");
                return;
            };

            let asset_amount_value = (*asset_amount).clone();
            let asset_amount_parsed: u64 = asset_amount_value.parse().unwrap_or(0);

            let min_amount_valid = match *selected_asset {
                SupportedAsset::LiquidBitcoin => asset_amount_parsed >= 1000,
                SupportedAsset::Tether => asset_amount_parsed >= 1,
            };

            if recipient_address_value.is_empty() {
                show_error_toast("Please enter a recipient address");
                return;
            }

            if asset_amount_parsed == 0 {
                show_error_toast("Please enter a valid amount");
                return;
            }

            if !min_amount_valid {
                let error_msg = match *selected_asset {
                    SupportedAsset::LiquidBitcoin => {
                        "Amount must be at least 1000 sats (0.00001 L-BTC)"
                    }
                    SupportedAsset::Tether => "Amount must be at least 1 USDT",
                };
                show_error_toast(error_msg);
                return;
            }

            let asset_id = selected_asset.get_asset_id();
            web_sys::console::log_1(
                &format!(
                    "Sending {} {} to {}",
                    asset_amount_parsed,
                    (*selected_asset).get_asset_info().2,
                    elements_address
                )
                .into(),
            );
            send_coins_with_modal.emit((elements_address, asset_amount_parsed, asset_id));
        })
    };

    let on_asset_change = {
        let selected_asset = selected_asset.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<web_sys::HtmlElement>();
            let js_obj: &JsValue = target.as_ref();
            if let Ok(value) = js_sys::Reflect::get(js_obj, &"value".into()) {
                if let Some(value_str) = value.as_string() {
                    match value_str.as_str() {
                        "L-BTC" => selected_asset.set(SupportedAsset::LiquidBitcoin),
                        "USDT" => selected_asset.set(SupportedAsset::Tether),
                        _ => {}
                    }
                }
            }
        })
    };

    let on_address_change = {
        let address_valid = address_valid.clone();
        let recipient_address = recipient_address.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let value = input.value();
            let is_valid = value.parse::<elements::Address>().is_ok();
            address_valid.set(Some(is_valid));
            recipient_address.set(value);
        })
    };

    let on_amount_change = {
        let asset_amount = asset_amount.clone();
        Callback::from(move |e: Event| {
            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let value = input.value();
            asset_amount.set(value);
        })
    };

    let go_to_home_and_sync = {
        Callback::from({
            let navigator = navigator.clone();
            let wallet_ctx = wallet_ctx.clone();
            let nostrades_db = nostrades_db.clone();
            let show_success_modal = show_success_modal.clone();

            move |_: MouseEvent| {
                show_success_modal.set(false);

                navigator.push(&crate::router::AppRoute::Home);

                let wallet = wallet_ctx.clone();
                let db = nostrades_db.clone();
                yew::platform::spawn_local(async move {
                    if wallet.update_wallet(&db).await.is_ok() {
                        wallet.dispatch(crate::contexts::NostradeWalletAction::Synced);
                        web_sys::console::log_1(&"Wallet synced after transaction".into());
                    }
                });
            }
        })
    };

    html!(
        <>
            <form {onsubmit}
                class="flex flex-col w-full max-w-md mx-auto shadow-xl rounded-2xl items-center gap-6 justify-evenly px-6 py-8 bg-card mt-16 border border-foreground/30"
            >
                <div class="w-full space-y-2">
                    <label class="block text-sm font-medium text-foreground">{"Select Asset"}</label>
                    <div class="flex items-center gap-3 rounded-lg bg-gray-50">
                        <img
                            src={asset_img}
                            class="size-12 hidden sm:block"
                            alt="Asset Icon" />
                        <div class="flex-1">
                            <select
                                name="asset_selector"
                                onchange={on_asset_change}
                                class="w-full p-3 border border-gray-300 rounded-lg bg-white focus:ring-2 focus:ring-primary focus:border-transparent"
                            >
                                <option value="L-BTC" selected={matches!(*selected_asset, SupportedAsset::LiquidBitcoin)}>
                                    {"Liquid Bitcoin (L-BTC)"}
                                </option>
                                <option value="USDT" selected={matches!(*selected_asset, SupportedAsset::Tether)}>
                                    {"Tether USDt (USDT)"}
                                </option>
                            </select>
                        </div>
                    </div>
                </div>

                <div class="w-full space-y-2">
                    <label class="block text-sm font-medium text-foreground">{"Recipient Address"}</label>
                    <div class="relative">
                        <input
                            type="text"
                            name="recipient_address"
                            value={(*recipient_address).clone()}
                            placeholder="Enter recipient address"
                            onchange={on_address_change}
                            class={format!(
                                "w-full py-3 px-4 border rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent {}",
                                match *address_valid {
                                    Some(true) => "border-primary/30 bg-muted",
                                    Some(false) => "border-destructive/30 bg-destructive/10",
                                    None => "border-gray-300"
                                }
                            )}
                            required=true />
                        {match *address_valid {
                            Some(false) => html! {
                                <p class="mt-1 text-sm text-destructive">{"Invalid address format"}</p>
                            },
                            Some(true) => html! {
                                <p class="mt-1 text-sm text-primary">{"Valid address"}</p>
                            },
                            None => html! {}
                        }}
                    </div>
                </div>

                <div class="w-full space-y-2">
                    <label class="block text-sm font-medium text-foreground">
                        {format!("Amount ({})", asset_symbol)}
                    </label>
                    <input
                        type="number"
                        name="asset_amount"
                        value={(*asset_amount).clone()}
                        placeholder={format!("Enter {asset_symbol} amount")}
                        onchange={on_amount_change}
                        step={if matches!(*selected_asset, SupportedAsset::LiquidBitcoin) { "0.00001" } else { "1" }}
                        min={if matches!(*selected_asset, SupportedAsset::LiquidBitcoin) { "0.00001" } else { "1" }}
                        class="w-full py-3 px-4 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
                        required=true
                    />
                    <p class="text-xs text-gray-500">
                        {if matches!(*selected_asset, SupportedAsset::LiquidBitcoin) {
                            "Minimum: 1000 sats (0.00001 L-BTC)"
                        } else {
                            "Minimum: 1 USDT"
                        }}
                    </p>
                </div>

                <button
                    type="submit"
                    class="w-full bg-primary text-white py-3 px-6 rounded-lg text-lg font-medium hover:cursor-pointer"
                >
                    {format!("Send {}", asset_name)}
                </button>
            </form>

            <Modal is_open={show_success_modal.clone()}>
                <div class="bg-white rounded-lg p-6 shadow-xl max-w-sm w-full">
                    <div class="flex justify-center mb-4">
                        <div class="size-16 bg-muted rounded-full border border-primary/30 flex items-center justify-center">
                            <svg class="size-8 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M5 13l4 4L19 7"></path>
                            </svg>
                        </div>
                    </div>

                    <h3 class="text-lg font-semibold text-center text-foreground mb-2">
                        {"Transaction Successful!"}
                    </h3>

                    <p class="text-sm text-muted-foreground text-center mb-4">
                        {format!("Your {} has been sent successfully.", asset_name)}
                    </p>

                    <div class="bg-gray-200 rounded-lg p-3 mb-4">
                        <p class="text-xs text-gray-500 mb-1">{"Transaction ID:"}</p>
                        <p class="text-sm text-gray-900 break-all">
                            {(*transaction_id).clone()}
                        </p>
                    </div>

                    <div class="mb-3">
                        <a
                            href={format!("https://liquid.network/liquidtestnet/tx/{}", (*transaction_id).clone())}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center justify-center w-full px-4 py-2 text-sm font-medium text-blue-600 bg-blue-50 border border-blue-200 rounded-lg hover:bg-blue-100 transition-colors hover:cursor-pointer"
                        >
                            {"Track Transaction"}
                            <svg class="w-4 h-4 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                            </svg>
                        </a>
                    </div>

                    <div class="flex flex-col gap-3">
                        <button
                            onclick={go_to_home_and_sync}
                            class="w-full bg-primary text-white py-2 px-4 rounded-lg font-medium hover:cursor-pointer transition-colors"
                        >
                            {"Go to Home & Sync Wallet"}
                        </button>
                        <button
                            onclick={let modal = show_success_modal.clone(); Callback::from(move |_| modal.set(false))}
                            class="w-full bg-gray-200 text-gray-800 py-2 px-4 rounded-lg font-medium hover:bg-gray-300 transition-colors hover:cursor-pointer"
                        >
                            {"Close"}
                        </button>
                    </div>
                </div>
            </Modal>
        </>
    )
}
