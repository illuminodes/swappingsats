use yew::prelude::*;

use crate::wallet_provider::ESPLORA_CLIENT;

#[function_component(OrderBookScreen)]
pub fn order_book_screen() -> HtmlResult {
    let relay_ctx = nostr_minions::use_nostr_relay_pool();
    let wallet_ctx = crate::wallet_provider::use_wallet_ctx();

    let validated_proposals = use_state(|| vec![]);
    let validated = use_state(|| vec![]);

    let relay_sender = relay_ctx.clone();
    use_effect_with((), move |()| {
        let filter = nostr_minions::nostro2::NostrSubscription {
            kinds: vec![32121].into(),
            ..Default::default()
        };
        let _ = relay_sender.send(filter);
    });

    let validated_setter = validated_proposals.clone();
    use_effect_with(relay_ctx.unique_notes.clone(), move |notes| {
        if let Some(note) = notes.last() {
            let Ok(proposal) = note
                .content
                .parse::<lwk_wollet::LiquidexProposal<lwk_wollet::Unvalidated>>()
            else {
                return;
            };
            let Ok(needed_tx) = proposal.needed_tx() else {
                return;
            };
            let mut validated_proposals = (*validated_setter).clone();
            validated_proposals.push((needed_tx, proposal));
            validated_setter.set(validated_proposals);
            // yew::platform::spawn_local(async move {
            //     let txs = ESPLORA_CLIENT
            //         .write()
            //         .await
            //         .get_transactions(&[proposal_data])
            //         .await
            //         .unwrap();
            //     let tx = txs.first().unwrap();
            //     let validated_proposal = proposal.validate(tx.clone()).expect("Failed to validate");
            //     let mut validated_proposals = (*validated_setter).clone();
            //     validated_proposals.push(validated_proposal);
            //     validated_setter.set(validated_proposals);
            // });
        }
    });
    let acquired_proposals = validated_proposals.clone();
    let really_validated = validated.clone();
    use_effect_with(relay_ctx.relay_events.clone(), move |notes| {
        if let Some(nostr_minions::nostro2::NostrRelayEvent::EndOfSubscription(..)) = notes.last() {
            web_sys::console::log_1(&"Order Book updated".into());
            yew::platform::spawn_local(async move {
                let tx_ids = acquired_proposals
                    .iter()
                    .map(|(id, _)| id)
                    .copied()
                    .collect::<Vec<_>>();
                let txs = ESPLORA_CLIENT
                    .write()
                    .await
                    .get_transactions(tx_ids.as_slice())
                    .await
                    .unwrap();
                let mut validated = vec![];
                for tx in txs {
                    let proposal = acquired_proposals
                        .iter()
                        .find(|(id, _)| *id == tx.txid())
                        .map(|(_, proposal)| proposal)
                        .cloned()
                        .unwrap();
                    let Ok(validated_proposal) = proposal.validate(tx.clone()) else {
                        continue;
                    };
                    validated.push(validated_proposal);
                }
                really_validated.set(validated);
            });
        }
    });
    web_sys::console::log_1(&format!("Proposals: {}", validated_proposals.len()).into());

    let onclick = {
        Callback::from(
            move |proposal: lwk_wollet::LiquidexProposal<lwk_wollet::Validated>| {
                let wallet = wallet_ctx.clone();
                yew::platform::spawn_local(async move {
                    if let Err(e) = wallet.liquidex_take(proposal).await {
                        web_sys::console::error_1(&format!("Failed to take: {e}").into());
                    }
                });
            },
        )
    };

    Ok(html! {
        <div class="p-4 flex flex-col gap-4 min-h-screen">
            <h1 class="font-semibold">{"Order Book"}</h1>
            {for validated.iter().map(|proposal| {
                let proposal = proposal.clone();
                html! {
                    <div class="flex items-center gap-4 justify-evenly px-6">
                             <div class="flex flex-col items-center gap-2">
                                 // <img src={proposal.input().asset.to_string()} alt={proposal.input().asset.to_string()} class="size-12" />
                                 <h3 class="font-semibold">{proposal.input().asset.to_string()[0..5].to_string()}</h3>
                                 <p class="text-gray-400">{format!("{}", proposal.input().amount)}</p>
                             </div>
                         <h3 class="text-lg font-semibold">{"Swap For"}</h3>
                             <div class="flex flex-col items-center gap-2">
                                 <h3 class="font-semibold">{proposal.output().asset.to_string()[0..5].to_string()}</h3>
                                 <p class="text-gray-400">{format!("{}", proposal.output().amount)}</p>
                             </div>
                             <button onclick={onclick.reform(move |_| {
                                proposal.clone()
                             })} class="mt-4 bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600">
                                {"Take"}
                             </button>
                    </div>
                }
            })}
        </div>

    })
}
