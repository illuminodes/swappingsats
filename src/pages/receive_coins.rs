use yew::prelude::*;
use yew_bitcoin_qr::BitcoinQrCode;

#[function_component(ReceiveCoinsScreen)]
pub fn receive_coins_screen() -> Html {
    let asset_handle = use_state(|| None::<SupportedAsset>);

    let go_back_button = {
        if asset_handle.is_some() {
            html! {
               <button
                   onclick={
                       Callback::from(move |_| {
                           asset_handle.set(None);
                       })
                   }
                   class="shadow-lg rounded-xl hover:cursor-pointer flex items-center justify-center"
                >
                   <crate::components::ArrowLeft class="size-5" />
               </button>
            }
        } else {
            html! {
               <yew_router::components::Link<crate::router::AppRoute>
                   to={crate::router::AppRoute::Home}>
                   <button class="shadow-lg rounded-xl flex items-center justify-center hover:cursor-pointer">
                       <crate::components::ArrowLeft class="size-5" />
                   </button>
               </yew_router::components::Link<crate::router::AppRoute>>
            }
        }
    };

    html! {
        <div class="flex flex-col items-center h-full max-w-4xl mx-auto flex-1 px-5 md:px-10 py-5">
            // Header
            <div class="flex items-center w-full gap-5">
                { go_back_button }
                <h2 class="text-2xl font-bold text-balance text-foreground">{"Receive"}</h2>
            </div>

            <div class="w-full h-fit mt-10">
                <ReceiveLiquidCoins />
            </div>
        </div>
    }
}

#[function_component(ReceiveLiquidCoins)]
fn receive_liquid_coins() -> Html {
    let copied = use_state(|| false);
    let Some(address) = crate::use_wallet_address() else {
        return html! {
            <div class="p-4">
                <p class="text-destructive">{"Failed to load wallet address."}</p>
            </div>
        };
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

    html! {
          <>
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
              <div class="bg-white rounded-lg mb-6 flex justify-center">
                  <div class="size-fit bg-white border-2 border-gray-200 rounded-lg flex items-center justify-center">
                    <BitcoinQrCode
                        id={"bitcoin-qr".to_string()}
                        address={address.to_string()}
                        size={400}
                        image_url={Some("https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcSuEvQLIjpwyNdMRB6Tt1vJzaEjFHiX_Xixng&s".to_string())}
                        dots_shape={yew_bitcoin_qr::Shape::Circle}
                        dots_color={"#006c36".to_string()}
                        corners_square_color={"#006c36".to_string()}
                        corners_dot_color={"#006c36".to_string()}
                    />

                  </div>
              </div>

              // Address
              <code {onclick}
                  class="flex items-center justify-evenly gap-2 bg-gray-200 border border-gray-400 shadow-sm p-2 rounded-lg mb-6 max-w-md mx-auto overflow-clip">
                  { copied_address }
              </code>

              // Action buttons
              <div class="flex flex-col md:flex-row max-w-md mx-auto gap-4">
                  <button class="flex-1 p-2 border border-gray-200 shadow-lg rounded-xl flex items-center justify-center">
                      <span class="mr-2">{"💰"}</span>
                      {"Set Amount"}
                  </button>
                  <button class="flex-1 p-2 border border-gray-200 shadow-lg rounded-xl flex items-center justify-center">
                      <crate::components::Share class="size-4 mr-2" />
                      {"Share"}
                  </button>
              </div>
          </>
    }
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
        let selected_asset = selected_asset;
        Callback::from(move |asset: SupportedAsset| {
            selected_asset.set(Some(asset));
        })
    };
    html!(
        <div class="flex flex-col gap-4">
            <div
                onclick={
                    onclick.reform(|_| SupportedAsset::LiquidBitcoin)
                }
                class="gap-2 bg-card border border-primary/40 hover:cursor-pointer shadow-xl p-4 rounded-xl h-fit bg-muted">
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
                class="gap-2 border border-primary/40 hover:cursor-pointer shadow-xl p-4 rounded-xl h-fit bg-muted">
                <img src="https://tether.to/images/logoCircle.png" class="size-14 my-2" />
                <h3 class="font-semibold">{"Tether"}</h3>
                <p class="text-sm text-gray-400">{"USDt"}</p>
            </div>
        </div>
    )
}
