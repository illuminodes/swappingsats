use yew_bitcoin_qr::BitcoinQrCode;
use yew::prelude::*;

#[function_component(ReceiveCoinsScreen)]
pub fn receive_coins_screen() -> Html {
    let asset_handle = use_state(|| None::<SupportedAsset>);

    let go_back_button = {
        let asset_handle = asset_handle.clone();
        if asset_handle.is_some() {
            html! {
               <button
                   onclick={
                       Callback::from(move |_| {
                           asset_handle.set(None);
                       })
                   }
                   class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
                   <crate::components::ArrowLeft size=5 />
               </button>
            }
        } else {
            html! {
               <yew_router::components::Link<crate::router::AppRoute>
                   to={crate::router::AppRoute::Home}>
                   <button class="p-2 border border-gray-200 shadow-lg mr-4 rounded-xl">
                       <crate::components::ArrowLeft size=5 />
                   </button>
               </yew_router::components::Link<crate::router::AppRoute>>
            }
        }
    };

    html! {
        <>
            // Header
            <div class="p-4 flex items-center w-full justify-between">
                { go_back_button }
                <h1 class="font-semibold align-center">{"Receive"}</h1>
                <button class="w-9"/>
            </div>

            <div class="px-4">
            {match *asset_handle {
                Some(SupportedAsset::Tether | SupportedAsset::LiquidBitcoin) => html!(
                    <ReceiveLiquidCoins />
                ),
                None => html!(
                    <AssetTiles asset_handle={asset_handle.clone()} />
                ),
            }}
            </div>
        </>
    }
}

#[function_component(ReceiveLiquidCoins)]
fn receive_liquid_coins() -> HtmlResult {
    let copied = use_state(|| false);
    let Some(address) = crate::wallet_provider::use_wallet_address() else {
        return Ok(html! {
            <div class="p-4">
                <p class="text-red-500">{"Failed to load wallet address."}</p>
            </div>
        });
    };
    let onclick = {
        let copied = copied.setter();
        let address = address.clone();
        Callback::from(move |_| {
            copied.set(true);
            nostr_minions::browser_api::clipboard_copy(&address.to_string());
            let copied = copied.clone();
            let _ = gloo::timers::callback::Timeout::new(2000, move || {
                copied.set(false);
            })
            .forget();
        })
    };

    let copied_address = if *copied {
        html! { <span class="text-xs text-gray-600">{"Copied!"}</span> }
    } else {
        html! {
            <>
            <p class="text-xs text-gray-600">
                {format!("{}...{}", &address.to_string()[0..16], &address.to_string()[address.to_string().len() - 16..])}
            </p>
            <crate::components::Copy class="size-4 mr-1" />
            </>
        }
    };

    Ok(html! {
          <div class="p-6">
              <div class="text-center mb-8">
                  <h2>
                      {"This is your "}
                      <span class="font-semibold">{"Liquid"}</span>
                  </h2>
                  <p class=" text-sm text-gray-600">{"receiving address."}</p>
                  <p class="text-gray-600">
                      {"Receive any Liquid assets here"}
                  </p>
              </div>

              // QR Code
              <div class="bg-white p-6 rounded-lg mb-6 flex justify-center">
                  <div class="w-48 h-48 bg-white border-2 border-gray-200 rounded-lg flex items-center justify-center">
                      <BitcoinQrCode
                          id={"wallet-receive".to_string()}
                          bitcoin={address.to_string()}
                          width={240.to_string()}
                          height={240.to_string()}
                          type_={Some(yew_bitcoin_qr::QrType::Svg)}
                          corners_square_type={Some(yew_bitcoin_qr::QrCornersSquareType::ExtraRounded)}
                          image={Some("https://liquid.net/_next/static/media/logo.28b5ba97.svg".to_string())}
                          corners_square_color={Some("#03A6A6".to_string())}
                          corners_dot_color={Some("#04BFAD".to_string())}
                          dots_type={Some(yew_bitcoin_qr::QrDotsType::ClassyRounded)}
                          dots_color={Some("#100940".to_string())}
                          poll_interval={Some(100_000)}
                      />
                  </div>
              </div>

              // Address
              <code {onclick}
                  class="flex items-center justify-evenly gap-2 bg-gray-200 border border-gray-400 shadow-sm p-2 rounded-lg mb-6 max-w-xs">
                  { copied_address }
              </code>

              // Action buttons
              <div class="flex gap-4">
                  <button class="flex-1 p-2 border border-gray-200 shadow-lg rounded-xl flex items-center justify-center">
                      <span class="mr-2">{"💰"}</span>
                      {"Set Amount"}
                  </button>
                  <button class="flex-1 p-2 border border-gray-200 shadow-lg rounded-xl flex items-center justify-center">
                      <crate::components::Share class="size-4 mr-2" />
                      {"Share"}
                  </button>
              </div>
          </div>
    })
}

#[derive(Clone, PartialEq)]
enum SupportedAsset {
    LiquidBitcoin,
    Tether,
}

#[derive(Properties, Clone, PartialEq)]
struct AssetTilesProps {
    pub asset_handle: UseStateHandle<Option<SupportedAsset>>,
}

#[function_component(AssetTiles)]
fn asset_tiles(props: &AssetTilesProps) -> Html {
    let selected_asset = props.asset_handle.clone();
    let onclick = {
        let selected_asset = selected_asset.clone();
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
