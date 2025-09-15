use yew::prelude::*;

#[hook]
pub fn use_nostrades_db() -> super::NostradesIdb {
    let ctx = use_context::<super::NostradesDbStore>().expect("No wallet context found");
    (*ctx).clone()
}
