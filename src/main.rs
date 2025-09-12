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
pub mod persister;
pub use persister::*;
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
                    <WalletProvider>
                    <OrderBookProvider>
                        <router::MainPages />
                        // <WalletLoad />
                        <WalletSync />
                    </OrderBookProvider>
                    </WalletProvider>
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
    let syncing = use_state(|| false);
    let sync_handle = syncing.setter();
    let ctx_clone = ctx.clone();
    let sync_clone = sync_handle.clone();
    // use_effect_with((), move |()| {
    //     let sync_handle = sync_clone.clone();
    //     yew::platform::spawn_local(async move {
    //         // loop {
    //         sync_handle.set(true);
    //         // // web_sys::console::log_1(&"Syncing wallet...".into());
    //         if ctx_clone.full_sync().await.is_ok() {
    //             ctx_clone.dispatch(NostradeWalletAction::Synced);
    //             web_sys::console::log_1(&"Wallet synced successfully".into());
    //         } else {
    //             web_sys::console::error_1(&"Failed to sync wallet".into());
    //         }
    //         sync_handle.set(false);
    //         //     gloo::timers::future::sleep(std::time::Duration::from_secs(180)).await;
    //         // }
    //     });
    // });
    let icon = if *syncing {
        html! { <components::LoaderIcon size=5 class="animate-spin" /> }
    } else {
        html! { <components::CheckIcon size=5 /> }
    };
    html! {
        <div
            onclick={
                let sync_handle = sync_handle.clone();
                Callback::from(move |_| {
                web_sys::console::log_1(&"Wallet sync clicked".into());
                let sync_handle = sync_handle.clone();
                let ctx = ctx.clone();
                yew::platform::spawn_local(async move {
                    sync_handle.set(true);
                    if ctx.full_sync().await.is_ok() {
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

#[function_component(WalletLoad)]
fn wallet_load() -> HtmlResult {
    let ctx = use_context::<NostradeWalletStore>().expect("No wallet context found");
    let clone_ctx = ctx.clone();
    let _synced: yew::suspense::UseFutureHandle<Result<_, NostradeWalletError>> =
        yew::suspense::use_future_with((), |_| async move {
            clone_ctx.load().await?;
            ctx.dispatch(NostradeWalletAction::Loaded);
            web_sys::console::log_1(&"Wallet loaded successfully".into());
            Ok(())
        })?;
    Ok(html! {})
}
