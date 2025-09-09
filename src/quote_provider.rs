use yew::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct CryptoPairs {
    pub trade_pair: String,
    pub price: f64,
    pub valid_from: u64,
    pub valid_until: u64,
    pub min_amount: u64,
    pub max_amount: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NostradeQuotes {
    last_quote: Option<(i64, CryptoPairs)>,
    pub accepted_quotes: std::collections::HashMap<String, String>,
}

pub enum NostradeQuoteAction {
    NewQuote((i64, CryptoPairs)),
    NewTransaction(nostr_minions::nostro2::NostrNote),
}

impl Reducible for NostradeQuotes {
    type Action = NostradeQuoteAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            NostradeQuoteAction::NewQuote(quote) => {
                web_sys::console::log_1(&format!("New quote received: {:?}", quote).into());
                std::rc::Rc::new(Self {
                    last_quote: Some(quote),
                    accepted_quotes: self.accepted_quotes.clone(),
                })
            }
            NostradeQuoteAction::NewTransaction(tx_note) => {
                web_sys::console::log_1(&format!("New transaction received: {:?}", tx_note).into());
                let tags = tx_note.tags.find_tags("req_id");
                let Some(tagged_id) = tags.first() else {
                    web_sys::console::error_1(&"Transaction note missing req_id tag".into());
                    return self;
                };
                let mut new_quotes = self.accepted_quotes.clone();
                new_quotes.insert(tagged_id.to_string(), tx_note.content);
                std::rc::Rc::new(Self {
                    last_quote: self.last_quote.clone(),
                    accepted_quotes: new_quotes,
                })
            }
        }
    }
}

pub type QuoteStore = UseReducerHandle<NostradeQuotes>;

#[function_component(QuotesProvider)]
pub fn quotes_provider(props: &yew::html::ChildrenProps) -> Html {
    let relay_ctx = nostr_minions::relay_pool::use_nostr_relay_pool();
    let ctx = use_reducer(|| NostradeQuotes {
        last_quote: None,
        accepted_quotes: std::collections::HashMap::new(),
    });

    let relay_clone = relay_ctx.clone();
    use_memo((), move |()| {
        let quote_filter = nostr_minions::nostro2::NostrSubscription {
            kinds: vec![11111, 21213].into(),
            ..Default::default()
        };
        relay_clone.send(quote_filter);
    });

    let ctx_clone = ctx.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            if note.kind == 11111 {
                if let Ok(quote) = serde_json::from_str::<CryptoPairs>(&note.content) {
                    ctx_clone.dispatch(NostradeQuoteAction::NewQuote((note.created_at, quote)));
                } else {
                    web_sys::console::error_1(
                        &format!("Failed to parse quote: {}", note.content).into(),
                    );
                }
            }
            if note.kind == 21213 {
                ctx_clone.dispatch(NostradeQuoteAction::NewTransaction(note.clone()));
            }
        }
        || {}
    });

    html! {
        <ContextProvider<QuoteStore> context={ctx}>
            {props.children.clone()}
        </ContextProvider<QuoteStore>>
    }
}

#[hook]
pub fn use_quotes() -> QuoteStore {
    use_context::<QuoteStore>().expect("No quotes context found")
}
#[hook]
pub fn use_quote() -> Option<(i64, CryptoPairs)> {
    let ctx = use_quotes();
    ctx.last_quote.clone()
}
