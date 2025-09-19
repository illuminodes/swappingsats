use yew::prelude::*;

#[derive(Properties, PartialEq, Clone, Eq)]
pub struct IconProps {
    #[prop_or(24)]
    pub size: u32,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(LoaderIcon)]
pub fn loader_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-loader-icon lucide-loader"><path d="M12 2v4"/><path d="m16.2 7.8 2.9-2.9"/><path d="M18 12h4"/><path d="m16.2 16.2 2.9 2.9"/><path d="M12 18v4"/><path d="m4.9 19.1 2.9-2.9"/><path d="M2 12h4"/><path d="m4.9 4.9 2.9 2.9"/></svg>
    }
}

#[function_component(CheckIcon)]
pub fn check_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-check-icon lucide-check"><path d="M20 6 9 17l-5-5"/></svg>
    }
}
#[function_component(ArrowLeft)]
pub fn spinner_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-left-icon lucide-arrow-left"><path d="m12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
    }
}

#[function_component(ArrowUpDown)]
pub fn spinner_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-up-down-icon lucide-arrow-up-down"><path d="M12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
    }
}

#[function_component(ArrowUpRight)]
pub fn spinner_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-up-right-icon lucide-arrow-up-right"><path d="M12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
    }
}

#[function_component(ArrowDownLeft)]
pub fn spinner_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-down-left-icon lucide-arrow-down-left"><path d="M12 19-7-7 7-7"/><path d="M19 12H5"/></svg>
    }
}

#[function_component(Copy)]
pub fn copy_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-copy-icon lucide-copy"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
    }
}
#[function_component(Share)]
pub fn share_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
            {class}
            width={size.to_string()}
            height={size.to_string()}
            viewBox={format!("0 0 {size} {size}")}
            fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-share-icon lucide-share"><path d="M12 2v13"/><path d="m16 6-4-4-4 4"/><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/></svg>
    }
}

#[function_component(History)]
pub fn history_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-history-icon lucide-history"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/></svg>
    }
}

#[function_component(Home)]
pub fn home_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        stroke-width="1.5" stroke="currentColor" fill="none">
        <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
        </svg>
    }
}

#[function_component(Settings)]
pub fn settings_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-settings-icon lucide-settings"><path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915"/><circle cx="12" cy="12" r="3"/></svg>
    }
}

#[function_component(OrderBook)]
pub fn order_book_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-book-open-icon lucide-book-open"><path d="M12 7v14"/><path d="M3 18a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h5a4 4 0 0 1 4 4 4 4 0 0 1 4-4h5a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1h-6a3 3 0 0 0-3 3 3 3 0 0 0-3-3z"/></svg>
    }
}

#[function_component(Keys)]
pub fn keys_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-key-round-icon lucide-key-round"><path d="M2.586 17.414A2 2 0 0 0 2 18.828V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.172a2 2 0 0 0 1.414-.586l.814-.814a6.5 6.5 0 1 0-4-4z"/><circle cx="16.5" cy="7.5" r=".5" fill="currentColor"/></svg>
    }
}

#[function_component(ArrowRightLeft)]
pub fn arrow_right_left_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-right-left-icon lucide-arrow-right-left"><path d="m16 3 4 4-4 4"/><path d="M20 7H4"/><path d="m8 21-4-4 4-4"/><path d="M4 17h16"/></svg>
    }
}

#[function_component(ArrowUp)]
pub fn arrow_up_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-up-icon lucide-arrow-up"><path d="m5 12 7-7 7 7"/><path d="M12 19V5"/></svg>
    }
}

#[function_component(ArrowDown)]
pub fn arrow_down_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-arrow-down-icon lucide-arrow-down"><path d="M12 5v14"/><path d="m19 12-7 7-7-7"/></svg>
    }
}

#[function_component(TrendingUp)]
pub fn trending_up_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();

    html! {
        <svg xmlns="http://www.w3.org/2000/svg"
        {class}
        width={size.to_string()}
        height={size.to_string()}
        viewBox={format!("0 0 {size} {size}")}
        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trending-up-icon lucide-trending-up"><path d="M3 17 9 11 13 15 21 7"/><path d="M14 7h7v7"/></svg>
    }
}
