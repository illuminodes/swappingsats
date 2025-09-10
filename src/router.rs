use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
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
        <BottomNavigation />
        </>
    }
}

#[function_component(BottomNavigation)]
pub fn bottom_navigation() -> HtmlResult {
    let current_route = use_route::<AppRoute>();
    let base_class = classes!("flex", "flex-col", "items-center", "py-3");
    Ok(html! {
    // Bottom Navigation
    <div class="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200">
          <div class="flex justify-around py-2">
              <yew_router::components::Link<crate::router::AppRoute>
                  to={crate::router::AppRoute::History}>
                  <button class={
                    classes!(base_class.clone(), if current_route == Some(AppRoute::History) { "text-blue-600" } else { "text-gray-500" })
                  }>
                      // <lucide_yew::History class="w-5 h-5 mb-1" />
                      <span class="text-xs">{"History"}</span>
                  </button>
              </yew_router::components::Link<crate::router::AppRoute>>
              <yew_router::components::Link<crate::router::AppRoute>
                  to={crate::router::AppRoute::Home}>
                  <button class={
                    classes!(base_class.clone(), if current_route == Some(AppRoute::Home) { "text-blue-600" } else { "text-gray-500" })
                  }>
                      // <lucide_yew::Wallet class="w-5 h-5 mb-1" />
                      <span class="text-xs">{"Wallets"}</span>
                  </button>
              </yew_router::components::Link<crate::router::AppRoute>>
              <yew_router::components::Link<crate::router::AppRoute>
                  to={crate::router::AppRoute::Keys}>
                  <button class={
                    classes!(base_class.clone(), if current_route == Some(AppRoute::Keys) { "text-blue-600" } else { "text-gray-500" })
                  }>
                      // <lucide_yew::Settings class="w-5 h-5 mb-1" />
                      <span class="text-xs">{"Settings"}</span>
                  </button>
              </yew_router::components::Link<crate::router::AppRoute>>
              <yew_router::components::Link<crate::router::AppRoute>
                  to={crate::router::AppRoute::OrderBook}>
                  <button class={
                    classes!(base_class.clone(), if current_route == Some(AppRoute::OrderBook) { "text-blue-600" } else { "text-gray-500" })
                  }>
                      // <lucide_yew::Settings class="w-5 h-5 mb-1" />
                      <span class="text-xs">{"Order Book"}</span>
                  </button>
              </yew_router::components::Link<crate::router::AppRoute>>
        </div>
    </div>
    })
}
