use std::str::FromStr;

use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportedAsset {
    Bitcoin,
    LiquidBitcoin,
    Tether,
}

#[function_component(SendCoinsScreen)]
pub fn send_coins_screen() -> Html {
    let selected_asset = use_state(|| None::<SupportedAsset>);
    let go_back_button = {
        let asset_handle = selected_asset.clone();
        if asset_handle.is_some() {
            html! {
               <button
                   onclick={
                       Callback::from(move |_| {
                           asset_handle.set(None);
                       })
                   }
                   class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
                   // <lucide_yew::ArrowLeft class="size-5" />
               </button>
            }
        } else {
            html! {
               <yew_router::components::Link<crate::router::AppRoute>
                   to={crate::router::AppRoute::Home}>
                   <button class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
                      //  <lucide_yew::ArrowLeft class="size-5" />
                   </button>
               </yew_router::components::Link<crate::router::AppRoute>>
            }
        }
    };

    html!(
        <>
            // Header
            <div class="p-4 flex items-center w-full justify-between">
                {go_back_button}
                <h1 class="font-semibold align-center">{"Send"}</h1>
                <button class="w-9"/>
            </div>
            <div class="p-6">
                {match *selected_asset {
                    Some(_) => html!(
                        <SendCoinForm asset_handle={selected_asset.clone()} />
                    ),
                    None => html!(
                        <AssetTiles asset_handle={selected_asset.clone()} />
                    ),
                }}
            </div>
        </>
    )
}

#[derive(Properties, Clone, PartialEq)]
struct AssetTilesProps {
    pub asset_handle: UseStateHandle<Option<SupportedAsset>>,
}

#[function_component(SendCoinForm)]
fn send_coin_form(props: &AssetTilesProps) -> Html {
    // Usa el hook existente que ya maneja el envío de coins
    let send_coins_cb = crate::use_send_coins();

    let (asset_img, asset_name) = match *props.asset_handle {
        Some(SupportedAsset::Bitcoin) => (
            "https://upload.wikimedia.org/wikipedia/commons/thumb/4/46/Bitcoin.svg/1200px-Bitcoin.svg.png",
            "Bitcoin",
        ),
        Some(SupportedAsset::LiquidBitcoin) => (
            "https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png",
            "Liquid Bitcoin",
        ),
        Some(SupportedAsset::Tether) => ("https://tether.to/images/logoCircle.png", "Tether"),
        None => ("", "Unknown Asset"),
    };
    let address = use_state(|| None::<bool>);
    let onsubmit = {
        let asset_handle = props.asset_handle.clone();
        let address = address.clone();
        let send_coins_cb = send_coins_cb;
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if address.is_none() || address.is_some_and(|addr| !addr) {
                web_sys::console::error_1(&"Invalid recipient address".into());
                return;
            }
            let form = e.target_unchecked_into::<web_sys::HtmlFormElement>();
            let recipient_address = form
                .get_with_name("recipient_address")
                .unwrap()
                .unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            let Ok(elements_address) = elements::Address::from_str(&recipient_address) else {
                web_sys::console::error_1(&"Invalid recipient address".into());
                return;
            };
            let asset_amount = form
                .get_with_name("asset_amount")
                .unwrap()
                .unchecked_into::<web_sys::HtmlInputElement>()
                .value();
            let asset_amount: u64 = asset_amount.parse().unwrap_or(0);
            if recipient_address.is_empty() || asset_amount == 0 {
                web_sys::console::error_1(&"Recipient address or asset amount is invalid".into());
                return;
            }
            if let Some(asset_id) = (*asset_handle).clone() {
                match asset_id {
                    SupportedAsset::Bitcoin => {
                        web_sys::console::log_1(
                            &format!("Sending {asset_amount} BTC to {elements_address}").into(),
                        );
                    }
                    SupportedAsset::LiquidBitcoin => {
                        let asset_id = *crate::T_L_BTC_ASSET_ID;
                        send_coins_cb.emit((elements_address, asset_amount, asset_id));
                    }
                    SupportedAsset::Tether => {
                        let asset_id = *crate::T_USDT_ASSET_ID;
                        send_coins_cb.emit((elements_address, asset_amount, asset_id));
                    }
                }
            }
        })
    };

    html!(
        <form {onsubmit}
            class="space-y-4">
            <img
                src={asset_img}
                class="w-16 h-16 mx-auto mb-4"
                alt="Asset Icon" />
            <h2 class="text-center font-semibold mb-2">{asset_name}</h2>
            <div class="relative w-full min-h-12">
                <input
                    type="text"
                    name="recipient_address"
                    placeholder="Recipient Address"
                    onchange={
                        let address = address.clone();
                        Callback::from(move |e: Event| {
                            let input = e.target_unchecked_into::<web_sys::HtmlInputElement>();
                            address.set(Some(input.value().parse::<elements::Address>().is_ok()));
                        })
                    }
                    class="absolute inset-0 p-2 border border-gray-200 rounded-lg max-w-64 truncate"
                    required=true />
                {match *address {
                    Some(false | true) | None => html!(),
                }}
            </div>
            <input
                type="number"
                name="asset_amount"
                placeholder="Enter Amount"
                class="w-full p-2 border border-gray-200 rounded-lg"
                required=true
                />
            <input
                type="submit"
                value="Send"
                class="w-full p-2 bg-blue-500 text-white rounded-lg cursor-pointer hover:bg-blue-600 mt-16"
                />
        </form>
    )
}

#[function_component(AssetTiles)]
fn asset_tiles(props: &AssetTilesProps) -> Html {
    let selected_asset = props.asset_handle.clone();
    let onclick = {
        let selected_asset = selected_asset;
        Callback::from(move |asset: SupportedAsset| {
            selected_asset.set(Some(asset));
        })
    };
    html!(
        <div class="grid grid-cols-2 gap-4">
            // Example asset tiles
            <div
                onclick={
                    onclick.reform(|_| SupportedAsset::LiquidBitcoin)
                }
                class="aspect-square gap-2 border border-gray-200 shadow-xl p-4 rounded-xl">
                <img
                    src="https://www.block-chain24.com/sites/default/files/crypto/liquid_network_l-btc_coin_icon.png"
                    class="size-14 my-2" />
                <h3 class="font-semibold">{"Liquid Bitcoin"}</h3>
                <p class="text-sm text-gray-400">{"L-BTC"}</p>
            </div>
            <div
                onclick={
                    onclick.reform(|_| SupportedAsset::Tether)
                }
                class="aspect-square gap-2 border border-gray-200 shadow-xl p-4 rounded-xl">
                <img src="https://tether.to/images/logoCircle.png" class="size-14 my-2" />
                <h3 class="font-semibold">{"Tether"}</h3>
                <p class="text-sm text-gray-400">{"USDt"}</p>
            </div>
        </div>
    )
}
