use yew::prelude::*;

pub type NostradesDbStore = UseStateHandle<super::NostradesIdb>;

#[function_component(NostradesDbProvider)]
pub fn provider(props: &yew::html::ChildrenProps) -> HtmlResult {
    let persistor = yew::suspense::use_future(|| async { crate::NostradesIdb::new().await })?;
    let Ok(persistor) = persistor.as_ref() else {
        return Ok(html! {
            {props.children.clone()}
        });
    };

    let ctx = use_state(|| persistor.clone());

    Ok(html! {
        <ContextProvider<NostradesDbStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<NostradesDbStore>>
    })
}
