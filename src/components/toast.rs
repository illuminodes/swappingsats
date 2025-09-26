use yew::prelude::*;
use yew_toasts::{Gravity, Position, ToastData, show_toast};

#[derive(Properties, PartialEq, Eq)]
pub struct SuccessToastProps {
    pub message: String,
}

#[function_component(SuccessToast)]
pub fn success_toast(props: &SuccessToastProps) -> Html {
    html! {
        <div class="bg-primary text-white font-semibold py-3 px-4 rounded-lg shadow-lg flex items-center space-x-2 max-w-sm">
            <svg class="size-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M5 13l4 4L19 7"></path>
            </svg>
            <span class="text-sm">{&props.message}</span>
        </div>
    }
}

#[derive(Properties, PartialEq, Eq)]
pub struct ErrorToastProps {
    pub message: String,
}

#[function_component(ErrorToast)]
pub fn error_toast(props: &ErrorToastProps) -> Html {
    html! {
        <div class="bg-destructive text-white font-semibold py-3 px-4 rounded-lg shadow-lg flex items-center space-x-2 max-w-sm">
            <svg class="size-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke_linecap="round" stroke_linejoin="round" stroke_width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
            <span class="text-sm">{&props.message}</span>
        </div>
    }
}

pub fn show_success_toast(message: &str) {
    let _ = show_toast(&ToastData {
        element: html!(<SuccessToast message={message.to_string()} />),
        position: Position::Center,
        gravity: Gravity::Top,
        duration: 4000,
        ..Default::default()
    });
}

pub fn show_error_toast(message: &str) {
    let _ = show_toast(&ToastData {
        element: html!(<ErrorToast message={message.to_string()} />),
        position: Position::Center,
        gravity: Gravity::Top,
        duration: 4000,
        ..Default::default()
    });
}
