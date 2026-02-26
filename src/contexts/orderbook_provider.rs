//! # Order Book Provider
//!
//! This module implements the real-time order book for SwappingSats, bridging the Nostr
//! protocol with the LiquiDEX atomic swap mechanism on the Liquid Network.
//!
//! ## How It Works
//!
//! ### Subscribing to Swap Offers
//!
//! On mount, [`OrderBookProvider`] opens a Nostr subscription for
//! **kind `32121`** events published in the last hour.  Every
//! [`nostr_minions::nostro2::NostrNote`] that arrives on the relay pool is
//! inspected:
//!
//! * If `content == "canceled"` → the offer identified by the note's first
//!   parameter tag (`"txid:vout"`) is removed from the local order book.
//! * If the content fails to parse as a
//!   [`lwk_wollet::LiquidexProposal<Unvalidated>`] → the note is silently
//!   ignored (wrong kind, spam, etc.).
//! * If `note.pubkey` matches the current user's public key → the offer is
//!   stored under `user_offer` (my own open orders).
//! * Otherwise → the offer is stored under `offers` (counterparty orders
//!   available to take).
//!
//! ### Validation Before Display
//!
//! Raw notes are stored **unvalidated** (the content is just JSON text).
//! Before showing offers in the UI — or allowing a user to take one — the
//! app calls [`OrderBook::parsed_offers`] (or [`OrderBook::my_offers`]).
//! These async methods:
//!
//! 1. Parse every stored note's `content` into a
//!    [`lwk_wollet::LiquidexProposal<Unvalidated>`].
//! 2. Ask LWK for the transaction ID the proposal depends on (`needed_tx()`).
//! 3. Batch-fetch those transactions from the Esplora API
//!    ([`crate::ESPLORA_CLIENT`]).
//! 4. Call `proposal.validate(tx)` via LWK, converting each proposal to
//!    [`lwk_wollet::LiquidexProposal<Validated>`].
//!
//! Only proposals that pass on-chain validation are returned to the caller.
//! This ensures the UI never shows an offer whose input UTXO has already
//! been spent.
//!
//! ### Nostr Event Format
//!
//! ```text
//! {
//!   "kind": 32121,
//!   "content": "<JSON-serialized LiquidexProposal>",
//!   "tags": [["param", "txid:vout"]],   // identifies the offered UTXO
//!   "pubkey": "<maker's hex pubkey>",
//!   ...
//! }
//! ```
//!
//! Cancellation events re-use the same kind and tag structure but set
//! `content` to the literal string `"canceled"`.

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

/// In-memory order book holding raw (unvalidated) Nostr notes.
///
/// Notes are stored as received from the relay pool.  Call
/// [`OrderBook::parsed_offers`] or [`OrderBook::my_offers`] to obtain
/// on-chain-validated proposals ready for display or acceptance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderBook {
    /// Swap offers published by the current user (my open orders).
    user_offer: Vec<nostr_minions::nostro2::NostrNote>,
    /// Swap offers published by other peers (available to take).
    offers: Vec<nostr_minions::nostro2::NostrNote>,
}

impl OrderBook {
    /// Returns all counterparty offers that pass on-chain validation.
    ///
    /// # Validation pipeline
    ///
    /// 1. Parse each stored note's `content` as a
    ///    [`lwk_wollet::LiquidexProposal<Unvalidated>`].  Notes whose content
    ///    cannot be parsed (e.g. stale data, wrong format) are silently dropped.
    /// 2. Call `proposal.needed_tx()` to obtain the Liquid transaction ID
    ///    that the proposal's input UTXO belongs to.
    /// 3. Batch-fetch all required transactions from the Esplora endpoint
    ///    ([`crate::ESPLORA_CLIENT`]).
    /// 4. For each proposal, find its transaction and call
    ///    `proposal.validate(tx)`.  Proposals whose UTXO has been spent or
    ///    whose amounts don't match the on-chain record are dropped.
    ///
    /// The returned pairs of `(note, validated_proposal)` are safe to display
    /// and accept.
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

    /// Returns the current user's own offers that still pass on-chain validation.
    ///
    /// Uses the same validation pipeline as [`OrderBook::parsed_offers`] but
    /// operates on `user_offer` instead of `offers`.  An offer that no longer
    /// validates (e.g. the UTXO was spent via a hard-cancel) will be absent
    /// from the result even if the Nostr event is still in memory.
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

/// Actions that can be dispatched to update the [`OrderBook`] reducer state.
pub enum OrderBookAction {
    /// A new offer from a peer was received from the relay pool.
    AddOffer(nostr_minions::nostro2::NostrNote),
    /// A new offer from the current user was received (echoed back by the relay).
    AddUserOffer(nostr_minions::nostro2::NostrNote),
    /// A cancellation event was received; removes the offer whose parameter tag
    /// matches the given `"txid:vout"` string.
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

/// Yew context provider that subscribes to the Nostr relay pool and populates
/// the [`OrderBook`] with live swap offers.
///
/// # Relay subscription
///
/// On first render, a [`nostr_minions::nostro2::NostrSubscription`] is sent to
/// the relay pool requesting all kind-`32121` events published within the last
/// **3600 seconds** (one hour).  Relays immediately replay matching stored
/// events and continue streaming new ones.
///
/// # Incoming event routing
///
/// Each time `relay_ctx.last_note` changes (a new event arrived), the
/// component inspects the note:
///
/// | Condition | Action |
/// |-----------|--------|
/// | `content == "canceled"` | Dispatch `RemoveOffer(param_tag)` |
/// | Content unparseable as `LiquidexProposal<Unvalidated>` | Ignore |
/// | `note.pubkey == user_pk` | Dispatch `AddUserOffer(note)` |
/// | Otherwise | Dispatch `AddOffer(note)` |
///
/// The [`OrderBook`] is available to descendant components via
/// [`use_orderbook_ctx`].
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
