use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct OrderBook {}

pub enum OrderBookAction {}

impl Reducible for OrderBook {
    type Action = OrderBookAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {}
    }
}


pub type OrderBookStore = UseReducerHandle<OrderBook>;

#[derive(serde::Deserialize)]
pub struct SpentResponse {
    pub spent: bool,
}

#[function_component(OrderBookProvider)]
pub fn quotes_provider(props: &yew::html::ChildrenProps) -> Html {
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let wallet_ctx = crate::use_wallet_ctx();
    let Some(nostr_key) = nostr_minions::use_nostr_key() else {
        return html! {};
    };
    let ctx = use_reducer(|| OrderBook {});
    let subscribed = use_mut_ref(|| None::<String>);
    // let unvalidated_proposals = use_mut_ref(Vec::new);

    let relay_clone = relay_ctx.clone();
    let sub_clone = subscribed.clone();
    let wallet_clone = wallet_ctx.clone();
    use_effect_with((), move |()| {
        let Some(persistor) = wallet_clone.persistor().cloned() else {
            return;
        };
        yew::platform::spawn_local(async move {
            let saved_offers = persistor.get_all_offers().await.unwrap();
            let latest_timestamp = saved_offers
                .iter()
                .filter_map(|offer| offer.offer.created_at.try_into().ok())
                .max()
                .map(|ts: u64| ts + 1);
            let quote_filter = nostr_minions::nostro2::NostrSubscription {
                kinds: vec![32121].into(),
                since: latest_timestamp,
                ..Default::default()
            };
            if let nostr_minions::nostro2::NostrClientEvent::Subscribe(_, id, _) =
                relay_clone.send(quote_filter)
            {
                *sub_clone.borrow_mut() = Some(id);
            }
        });
    });

    let user_pk = nostr_key.public_key();
    // let proposals_clone = unvalidated_proposals.clone();
    use_effect_with(relay_ctx.last_note.clone(), move |note| {
        let Some(note) = note else {
            return;
        };
        if user_pk == note.pubkey {
            return;
        }
        let Ok(liquidex_offer) = note
            .content
            .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
        else {
            return;
        };
        let Ok(needed_tx) = liquidex_offer.needed_tx() else {
            return;
        };
        let note_clone = note.clone();
        let Some(persistor) = wallet_ctx.persistor().cloned() else {
            return;
        };
        yew::platform::spawn_local(async move {
            let persisted_offer = crate::PersistedOffer::new(note_clone)
                .expect("Failed to create persisted offer");
            persistor
                .push_offer(persisted_offer)
                .await
                .expect("Failed to persist offer");
            // let tx = crate::ESPLORA_CLIENT
            //     .write()
            //     .await
            //     .get_transactions(&[needed_tx])
            //     .await
            //     .expect("");
            // let [tx] = tx.as_slice() else {
            //     return;
            // };
            // let tx_id = tx.txid();
            // let input_vout = &tx.input.first().unwrap().previous_output.vout;
            // let spent = MEMPOOL_CLIENT
            //     .get(format!(
            //         "https://liquid.network/api/tx/{tx_id}/outspend/{input_vout}"
            //     ))
            //     .send()
            //     .await
            //     .expect("Failed to get tx spend");
            // let spent: SpentResponse = spent.json().await.expect("Failed to parse tx spend");
            // if !spent.spent {}
        });
    });
    use_effect_with(relay_ctx.last_event.clone(), move |event| {
        if let Some(nostr_minions::nostro2::NostrRelayEvent::EndOfSubscription(.., id)) = event {
            if Some(id) == subscribed.borrow().as_ref() {
                //  web_sys::console::log_1(&"End of subscription".into());
                //  web_sys::console::log_1(
                //      &format!(
                //          "Unvalidated Proposals: {}",
                //          unvalidated_proposals.borrow().len()
                //      )
                //      .into(),
                //  );
                //  let unvalidated_proposals = unvalidated_proposals.borrow().clone();
                //  yew::platform::spawn_local(async move {
                //      let tx_ids = unvalidated_proposals
                //          .iter()
                //          .map(|(id, _)| id)
                //          .copied()
                //          .collect::<Vec<_>>();
                //      let txs = crate::ESPLORA_CLIENT
                //          .write()
                //          .await
                //          .get_transactions(tx_ids.as_slice())
                //          .await
                //          .unwrap();
                //  });
            }
        }
    });
    html! {
        <ContextProvider<OrderBookStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<OrderBookStore>>
    }
}
//
// #[hook]
// pub fn use_quotes() -> OrderBookStore {
//     use_context::<OrderBookStore>().expect("No quotes context found")
// }
// #[hook]
// pub fn use_quote() -> Option<(i64, CryptoPairs)> {
//     let ctx = use_quotes();
//     ctx.last_quote.clone()
// }
