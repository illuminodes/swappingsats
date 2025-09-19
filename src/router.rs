use crate::components::{History, Home, Keys, OrderBook};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Eq)]
pub enum AppRoute {
    #[at("/")]
    Home,
    #[at("/recieve")]
    Receive,
    #[at("/send")]
    SendCoins,
    #[at("/swap")]
    Swap,
    #[at("/keys")]
    Keys,
    #[at("/history")]
    History,
    #[at("/orderbook")]
    OrderBook,
}

#[function_component(MainPages)]
pub fn main_pages() -> Html {
    html! {
        <>
            <div class="py-5 border-b border-gray-300">
                <div class="max-w-7xl mx-auto">
                    <h1 class="text-4xl text-gray-700 font-black">{"Nostrades Wallet"}</h1>
                </div>
            </div>

            <div class="md:flex md:h-full">
                <Sidebar />
                <Switch<AppRoute> render={move |switch| {
                    match switch {
                        AppRoute::Home => html! { <crate::DashboardScreen /> },
                        AppRoute::Receive => html! { <crate::ReceiveCoinsScreen /> },
                        AppRoute::SendCoins => html! { <crate::SendCoinsScreen /> },
                        AppRoute::Swap => html! {
                            <>
                                <crate::SwapCoinsScreen />
                            </>
                        },
                        AppRoute::Keys => html! { <></> },
                        AppRoute::History => html! { <crate::WalletHistoryScreen /> },
                        AppRoute::OrderBook => html! { <crate::OrderBookScreen /> },
                    }
                }} />
            </div>

            <BottomNavigation />
        </>
    }
}

#[function_component(BottomNavigation)]
pub fn bottom_navigation() -> Html {
    let current_route = use_route::<AppRoute>();
    let base_class = classes!(
        "flex",
        "flex-col",
        "items-center",
        "p-3",
        "gap-2",
        "hover:cursor-pointer"
    );
    html! {
        // Bottom Navigation
        <div class="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 md:hidden">
            <div class="flex justify-around">
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::History}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::History) { "text-primary bg-muted" } else { "text-gray-600 hover:bg-primary hover:text-white transition-colors ease-in-out duration-300" })
                    }>
                        <History class="size-5" />
                        <span class="text-xs">{"History"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::Home}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::Home) { "text-primary bg-muted" } else { "text-gray-600 hover:bg-primary hover:text-white transition-colors ease-in-out duration-300" })
                    }>

                        <Home class="size-5" />
                        <span class="text-xs">{"Home"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::Keys}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::Keys) { "text-primary bg-muted" } else { "text-gray-600 hover:bg-primary hover:text-white transition-colors ease-in-out duration-300" })
                    }>
                        <Keys class="size-5" />
                        <span class="text-xs">{"Keys"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::OrderBook}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::OrderBook) { "text-primary bg-muted" } else { "text-gray-600 hover:bg-primary hover:text-white transition-colors ease-in-out duration-300" })
                    }>
                        <OrderBook class="size-5" />
                        <span class="text-xs">{"Order Book"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
            </div>
        </div>
    }
}

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let current_route = use_route::<AppRoute>();
    let base_class = classes!(
        "flex",
        "items-center",
        "px-3",
        "py-4",
        "gap-3",
        "hover:cursor-pointer",
        "rounded-lg",
        "w-full"
    );
    html! {
        <div class="h-full py-3 px-5 border-r border-gray-300 hidden md:block w-64">
            <div class="w-full flex flex-col gap-3">
                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::History}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::History) { "text-white bg-primary text-center" } else { "text-muted-foreground hover:bg-muted transition-colors ease-in-out duration-300" })
                    }>
                        <History class="size-5" />
                        <span class="text-md">{"History"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>

                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::Home}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::Home) { "text-white bg-primary text-center" } else { "text-muted-foreground hover:bg-muted transition-colors ease-in-out duration-300" })
                    }>
                        <Home class="size-5" />
                        <span class="text-md">{"Home"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>

                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::Keys}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::Keys) { "text-white bg-primary text-center" } else { "text-muted-foreground hover:bg-muted transition-colors ease-in-out duration-300" })
                    }>
                        <Keys class="size-5" />
                        <span class="text-md">{"Keys"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>

                <yew_router::components::Link<crate::router::AppRoute>
                    to={crate::router::AppRoute::OrderBook}>
                    <button class={
                        classes!(base_class.clone(), if current_route == Some(AppRoute::OrderBook) { "text-white bg-primary text-center" } else { "text-muted-foreground hover:bg-muted transition-colors ease-in-out duration-300" })
                    }>
                        <OrderBook class="size-5" />
                        <span class="text-md">{"OrderBook"}</span>
                    </button>
                </yew_router::components::Link<crate::router::AppRoute>>
            </div>
        </div>
    }
}
