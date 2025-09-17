use yew::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum OrderbookError {
    #[error("Nostr error: {0}")]
    Esplora(#[from] lwk_wollet::Error),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TxResponse {
    pub txid: elements::Txid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderBook {
    user_offer: Vec<nostr_minions::nostro2::NostrNote>,
    offers: Vec<nostr_minions::nostro2::NostrNote>,
}

impl OrderBook {
    pub async fn parsed_offers(
        &self,
    ) -> Result<
        Vec<(
            nostr_minions::nostro2::NostrNote,
            lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
        )>,
        OrderbookError,
    > {
        let unvalidated_proposals = self
            .offers
            .iter()
            .filter_map(|n| {
                let proposal = n
                    .content
                    .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
                    .ok()?;
                let needed = proposal.needed_tx().ok()?;
                Some((n.clone(), proposal, needed))
            })
            .collect::<Vec<_>>();
        let needed = unvalidated_proposals
            .iter()
            .map(|(.., id)| id)
            .copied()
            .collect::<Vec<_>>();
        let txs = crate::ESPLORA_CLIENT
            .read()
            .await
            .get_transactions(needed.as_slice())
            .await?;
        let validated = unvalidated_proposals
            .iter()
            .cloned()
            .filter_map(|(n, proposal, needed)| {
                let tx = txs.iter().find(|tx| tx.txid() == needed)?.clone();
                let validated = proposal.validate(tx).ok()?;
                Some((n, validated))
            })
            .collect::<Vec<_>>();
        Ok(validated)
    }

    pub async fn my_offers(
        &self,
    ) -> Result<
        Vec<(
            nostr_minions::nostro2::NostrNote,
            lwk_wollet::LiquidexProposal<lwk_wollet::Validated>,
        )>,
        OrderbookError,
    > {
        let unvalidated_proposals = self
            .user_offer
            .iter()
            .filter_map(|n| {
                let proposal = n
                    .content
                    .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
                    .ok()?;
                let needed = proposal.needed_tx().ok()?;
                Some((n, proposal, needed))
            })
            .collect::<Vec<_>>();
        let txs = crate::ESPLORA_CLIENT
            .read()
            .await
            .get_transactions(
                unvalidated_proposals
                    .iter()
                    .map(|(.., id)| id)
                    .copied()
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .await?;
        let validated = unvalidated_proposals
            .iter()
            .cloned()
            .filter_map(|(n, proposal, needed)| {
                let tx = txs.iter().find(|tx| tx.txid() == needed)?.clone();
                let validated = proposal.validate(tx).ok()?;
                Some((n.clone(), validated))
            })
            .collect::<Vec<_>>();
        Ok(validated)
    }
}

pub enum OrderBookAction {
    AddOffer(nostr_minions::nostro2::NostrNote),
    AddUserOffer(nostr_minions::nostro2::NostrNote),
    RemoveOffer(String),
}

impl Reducible for OrderBook {
    type Action = OrderBookAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            OrderBookAction::AddOffer(offer) => {
                let mut offers = self.offers.clone();
                offers.push(offer);
                std::rc::Rc::new(Self {
                    user_offer: self.user_offer.clone(),
                    offers,
                })
            }
            OrderBookAction::AddUserOffer(offer) => {
                let mut user_offer = self.user_offer.clone();
                user_offer.push(offer);
                std::rc::Rc::new(Self {
                    user_offer,
                    offers: self.offers.clone(),
                })
            }
            OrderBookAction::RemoveOffer(offer_id) => {
                let mut offers = self.offers.clone();
                offers.retain(|o| o.tags.first_parameter().as_ref() != Some(&offer_id));
                std::rc::Rc::new(Self {
                    user_offer: self.user_offer.clone(),
                    offers,
                })
            }
        }
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
    let Some(nostr_key) = nostr_minions::use_nostr_key() else {
        return html! {};
    };
    let ctx = use_reducer(|| OrderBook {
        user_offer: Vec::new(),
        offers: Vec::new(),
    });
    let subscribed = use_mut_ref(|| None::<String>);

    let relay_clone = relay_ctx.clone();
    let sub_clone = subscribed;
    use_effect_with((), move |()| {
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_possible_truncation)]
        let last_hour = (web_sys::js_sys::Date::now() / 1000.0).abs().round() as u64 - 3600;
        let quote_filter = nostr_minions::nostro2::NostrSubscription {
            kinds: vec![32121].into(),
            since: Some(last_hour),
            ..Default::default()
        };
        if let nostr_minions::nostro2::NostrClientEvent::Subscribe(_, id, _) =
            relay_clone.send(quote_filter)
        {
            *sub_clone.borrow_mut() = Some(id);
        }
    });

    let user_pk = nostr_key.public_key();
    let dispatch = ctx.dispatcher();
    use_effect_with(relay_ctx.last_note.clone(), move |note| {
        let Some(note) = note else {
            return;
        };
        if note.content == "canceled" {
            if let Some(offer_id) = note.tags.first_parameter() {
                dispatch.dispatch(OrderBookAction::RemoveOffer(offer_id));
            }
            return;
        }
        if note
            .content
            .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
            .is_err()
        {
            return;
        }
        if user_pk == note.pubkey {
            dispatch.dispatch(OrderBookAction::AddUserOffer(note.clone()));
        } else {
            dispatch.dispatch(OrderBookAction::AddOffer(note.clone()));
        }
    });

    html! {
        <ContextProvider<OrderBookStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<OrderBookStore>>
    }
}

#[hook]
pub fn use_orderbook_ctx() -> OrderBookStore {
    use_context::<OrderBookStore>().expect("No orderbook context found")
}
