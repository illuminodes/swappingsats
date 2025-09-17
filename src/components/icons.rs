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
pub fn spinner_icon(props: &IconProps) -> Html {
    let IconProps { size, class } = props.clone();
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" {class} width={size.to_string()} height={size.to_string()} viewBox={format!("0 0 {size} {size}")} fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-loader-icon lucide-loader"><path d="M12 2v4"/><path d="m16.2 7.8 2.9-2.9"/><path d="M18 12h4"/><path d="m16.2 16.2 2.9 2.9"/><path d="M12 18v4"/><path d="m4.9 19.1 2.9-2.9"/><path d="M2 12h4"/><path d="m4.9 4.9 2.9 2.9"/></svg>
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
