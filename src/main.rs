#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::nursery,
    clippy::style
)]
#![allow(clippy::future_not_send)]

pub mod components;

use yew::prelude::*;
mod pages;
// mod quote_provider;
mod contexts;
mod router;
pub use contexts::*;
pub use pages::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let relays = vec![
        nostr_minions::UserRelay {
            url: "wss://no.str.cr".to_string(),
            read: true,
            write: true,
        },
        nostr_minions::UserRelay {
            url: "wss://relay.illuminodes.com".to_string(),
            read: true,
            write: true,
        },
    ];
    html! {
        <yew_router::BrowserRouter>
            <nostr_minions::NostrAppProvider {relays} fallback={html!{<SplashScreen />}}>
                <LoginCheck>
                <NostradesDbProvider>
                    <WalletProvider>
                    <OrderBookProvider>
                        <router::MainPages />
                        // <WalletLoad />
                        <WalletSync />
                    </OrderBookProvider>
                    </WalletProvider>
                    </NostradesDbProvider>
                </LoginCheck>
            </nostr_minions::NostrAppProvider>
        </yew_router::BrowserRouter>
    }
}

#[function_component(SplashScreen)]
fn splash_screen() -> Html {
    html! {
        <div class="w-screen h-screen bg-black flex items-center justify-center" >
            <img
            class={classes!("mb-4", "mx-auto", "size-48")}
            src="public/nostrtradeslogo.svg"
            alt="Nostrades Logo" />
        </div>
    }
}

#[function_component(LoginCheck)]
fn login_check(props: &yew::html::ChildrenProps) -> Html {
    let key_ctx = nostr_minions::use_nostr_key();
    if key_ctx.is_some() {
        props.children.clone()
    } else {
        html! {
            <div class="flex flex-col items-center justify-evenly h-screen w-screen p-4">
                <pages::NostrLogin />
            </div>
        }
    }
}

#[function_component(WalletSync)]
fn wallet_load() -> Html {
    let ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let nostrades_db = crate::use_nostrades_db();
    let syncing = use_state(|| false);
    let sync_handle = syncing.setter();
    let ctx_clone = ctx.clone();
    let sync_clone = sync_handle.clone();
    let db = nostrades_db.clone();
    use_effect_with((), move |()| {
        let sync_handle = sync_clone.clone();
        yew::platform::spawn_local(async move {
            // loop {
            sync_handle.set(true);
            // // web_sys::console::log_1(&"Syncing wallet...".into());
            if ctx_clone.update_wallet(&db).await.is_ok() {
                ctx_clone.dispatch(NostradeWalletAction::Synced);
                web_sys::console::log_1(&"Wallet synced successfully".into());
            } else {
                web_sys::console::error_1(&"Failed to sync wallet".into());
            }
            sync_handle.set(false);
            //     gloo::timers::future::sleep(std::time::Duration::from_secs(180)).await;
            // }
        });
    });
    let icon = if *syncing {
        html! { <components::LoaderIcon size=5 class="animate-spin" /> }
    } else {
        html! { <components::CheckIcon size=5 /> }
    };
    html! {
        <div
            onclick={
                let sync_handle = sync_handle.clone();
                let db = nostrades_db.clone();
                Callback::from(move |_| {
                web_sys::console::log_1(&"Wallet sync clicked".into());
                let sync_handle = sync_handle.clone();
                let ctx = ctx.clone();
                let trades_db = db.clone();
                yew::platform::spawn_local(async move {
                    sync_handle.set(true);
                    if ctx.update_wallet(&trades_db).await.is_ok() {
                        ctx.dispatch(NostradeWalletAction::Synced);
                        web_sys::console::log_1(&"Wallet synced successfully".into());
                    } else {
                        web_sys::console::error_1(&"Failed to sync wallet".into());
                    }
                    sync_handle.set(false);
                });
            })}
            class="fixed top-0 right-0 m-4 p-2 border border-gray-200 rounded-lg shadow-lg">
            {icon}
        </div>
    }
}

pub static T_L_BTC_ASSET_ID: std::sync::LazyLock<elements::AssetId> =
    std::sync::LazyLock::new(|| {
        "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49"
            .parse::<elements::AssetId>()
            .expect("Failed to parse asset id")
    });

pub static T_USDT_ASSET_ID: std::sync::LazyLock<elements::AssetId> =
    std::sync::LazyLock::new(|| {
        "38fca2d939696061a8f76d4e6b5eecd54e3b4221c846f24a6b279e79952850a5"
            .parse::<elements::AssetId>()
            .expect("Failed to parse asset id")
    });
