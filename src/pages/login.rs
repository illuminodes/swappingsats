use nostr_minions::nostro2::NostrSigner;
use shady_minions::ui::{
    Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Form, Input, Modal, Tabs,
    TabsContent, TabsList, TabsTrigger,
};
use web_sys::wasm_bindgen::JsCast;
use yew::prelude::*;
#[function_component(NostrLogin)]
pub fn login_form() -> Html {
    let open_modal = use_state(|| false);
    let login_modal = use_state(|| false);
    let onclick = {
        let modal = open_modal.clone();
        Callback::from(move |_| {
            modal.set(!(*modal));
        })
    };
    let login_onclick = {
        let modal = login_modal.clone();
        Callback::from(move |_| {
            modal.set(!(*modal));
        })
    };
    html! {
        <>
        <img
            class={classes!("mb-4", "mx-auto", "size-48")}
            src="public/nostrtradeslogo.svg"
            alt="Nostrades Logo" />
        <Card class={classes!("max-w-xs", "border-gray-200", "shadow-xl")}>
            <CardHeader>
                <CardTitle>{ "SwappingSats" }</CardTitle>
                <CardDescription>{ "Self-custodial atomic swaps." }</CardDescription>
            </CardHeader>
            <CardContent>
                <Tabs default_value="register" class={classes!("w-full")}>
                    <TabsList class={classes!("justify-stretch", "w-full", "flex")}>
                        <TabsTrigger value="register">{ "New Identity" }</TabsTrigger>
                        <TabsTrigger value="login">{ "Recover" }</TabsTrigger>
                    </TabsList>
                    <TabsContent value="register">
                        <p class={classes!("text-sm", "text-muted-foreground")}>
                            { "If you don't have a key, you can generate a new one here." }
                        </p>
                        <Button
                            {onclick}
                            r#type={shady_minions::ui::ButtonType::Button}
                            class={classes!("mt-4", "flex-1", "bg-blue-500", "text-white")}>
                            { "Generate Key" }
                        </Button>
                    </TabsContent>
                    <TabsContent value="login">
                        <p class={classes!("text-sm", "text-muted-foreground")}>
                            { "Recover your data from the cloud using your secret key." }
                        </p>
                        <Button
                            onclick={login_onclick}
                            r#type={shady_minions::ui::ButtonType::Button}
                            class={classes!("mt-4", "flex-1", "bg-blue-500", "text-white")}>
                            { "Input Key" }
                        </Button>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
        <Modal is_open={open_modal} >
            <NewKeyForm />
        </Modal>
        <Modal is_open={login_modal} >
            <LoginForm />
        </Modal>
        </>
    }
}
#[function_component(LoginForm)]
pub fn new_key_form() -> Html {
    // let nostr_store = nostr_minions::use_idb_database();
    let create_key_cb = nostr_minions::use_create_local_key();
    let mnemonic_submit = {
        // let nostr_store_clone = nostr_store.clone();
        let create_cb = create_key_cb.clone();
        Callback::from(move |form: web_sys::HtmlFormElement| {
            let mut mnemonic = vec![];
            for i in 1..=24 {
                let Some(word) = form
                    .get_with_name(&format!("word-{i}"))
                    .map(|input| input.unchecked_into::<web_sys::HtmlInputElement>().value())
                else {
                    web_sys::console::log_1(&"Error: Input not found".into());
                    return;
                };
                mnemonic.push(word);
            }
            let mnemonic = mnemonic.join(" ");
            let new_key = nostr_minions::nostro2_signer::keypair::NostrKeypair::parse_mnemonic(
                &mnemonic,
                nostr_minions::nostro2_signer::Language::English,
                true,
            )
            .expect("Failed to create new key");
            create_cb.emit(new_key);
        })
    };
    let nsec_submit = {
        let create_cb = create_key_cb.clone();
        Callback::from(move |form: web_sys::HtmlFormElement| {
            let Some(input) = form
                .get_with_name("hex-key")
                .map(|input| input.unchecked_into::<web_sys::HtmlInputElement>().value())
            else {
                web_sys::console::log_1(&"Error: Input not found".into());
                return;
            };
            let Ok(mut new_key) =
                input.parse::<nostr_minions::nostro2_signer::keypair::NostrKeypair>()
            else {
                // TODO
                web_sys::console::log_1(&"Error: Invalid NSEC key".into());
                return;
            };
            new_key.set_extractable(true);
            create_cb.emit(new_key);
        })
    };
    html! {
        <Card class={classes!("max-w-xs", "border-gray-200", "shadow-xl" , "bg-white")}>
            <CardHeader>
                <CardTitle>{"Log In"}</CardTitle>
                <CardDescription class={classes!("flex-1")}>
                    {
                        "
                            Your data is stored encrypted in the Nostr network.
                            It can only be recovered using your secret key.
                        "
                    }
                </CardDescription>
            </CardHeader>
            <CardContent class={classes!("space-y-4")}>
                <Tabs default_value="mnemonic" class={classes!("w-full")}>
                    <TabsList class={classes!("justify-stretch", "w-full", "flex")}>
                        <TabsTrigger value="mnemonic">{ "Seed Phrase" }</TabsTrigger>
                        <TabsTrigger value="hex-key">{ "Passkey" }</TabsTrigger>
                    </TabsList>
                    <TabsContent value="mnemonic" class={classes!("space-y-4")}>
                        <Form onsubmit={mnemonic_submit}>
                            <p class={classes!("font-bold", "text-muted-foreground", "select-none", "pointer-events-none")}>
                                { "Ensure you keep the correct order." }
                            </p>
                            <div class={classes!("font-bold", "text-sm", "grid", "grid-cols-3", "gap-2")}>
                                { (1..25).map(|i| {
                                    html! {
                                            <input
                                                id={format!("word-{i}")}
                                                placeholder={format!("Word {i}")}
                                                required={true}
                                                // r#type={shady_minions::ui::InputType::Text}
                                                class={classes!("text-sm", "font-bold", "text-center")}/>
                                    }
                                }).collect::<Html>() }
                            </div>
                            <Button
                                r#type={shady_minions::ui::ButtonType::Submit}
                                class={classes!("mt-4", "mr-4")}>
                                { "Save" }
                            </Button>
                        </Form>
                    </TabsContent>
                    <TabsContent value="hex-key" class={classes!("space-y-4")}>
                        <Form onsubmit={nsec_submit}>
                        <p class={classes!("font-bold", "text-muted-foreground", "select-none", "pointer-events-none")}>
                            { "Your secret key should start with the prefix `nsec`" }
                        </p>
                        <Input
                            id="hex-key"
                            placeholder="nsec_1234567890abcdef"
                            required={true}
                            r#type={shady_minions::ui::InputType::Password}
                            class={classes!("text-sm", "font-bold", "text-center")}/>
                        <Button
                            r#type={shady_minions::ui::ButtonType::Submit}
                            class={classes!("mt-4", "mr-4")}>
                            { "Save" }
                        </Button>
                        </Form>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    }
}
#[function_component(NewKeyForm)]
pub fn new_key_form() -> Html {
    let create_key_cb = nostr_minions::use_create_local_key();
    let new_key =
        use_state(|| nostr_minions::nostro2_signer::keypair::NostrKeypair::generate(true));
    let mnemonic = new_key
        .mnemonic(nostr_minions::nostro2_signer::Language::English)
        .unwrap_or_default();
    let hex_key = new_key.nsec().unwrap_or_default();
    let onclick = {
        let cb_create_key = create_key_cb.clone();
        let new_key = new_key.clone();
        Callback::from(move |_| {
            cb_create_key.emit((*new_key).clone());
        })
    };
    let copy_key = {
        let hex_keys = hex_key.clone();
        Callback::from(move |_| {
            nostr_minions::browser_api::clipboard_copy(&hex_keys);
        })
    };
    let generate_new_key = {
        let keys = new_key.setter();
        Callback::from(move |_| {
            let new_key = nostr_minions::nostro2_signer::keypair::NostrKeypair::generate(true);
            keys.set(new_key);
        })
    };
    html! {
        <Card class={classes!("max-w-xs", "border-gray-200", "shadow-xl" , "bg-white")}>
            <CardHeader>
                <CardTitle>{ "New Key" }</CardTitle>
                <CardDescription class={classes!("flex-1")}>
                    { "
                        This key is secret.
                        It cannot be recovered if lost.
                        We recommend using at least one of the following security options.
                      " 
                    }
                </CardDescription>
            </CardHeader>
            <CardContent class={classes!("space-y-4")}>
                <Tabs default_value="mnemonic" class={classes!("w-full")}>
                    <TabsList class={classes!("justify-stretch", "w-full", "flex")}>
                        <TabsTrigger value="mnemonic">{ "Seed Phrase" }</TabsTrigger>
                        <TabsTrigger value="hex-key">{ "Passkey" }</TabsTrigger>
                    </TabsList>
                    <TabsContent value="mnemonic" class={classes!("space-y-4")}>
                        <p class={classes!("font-bold", "text-muted-foreground", "select-none", "pointer-events-none")}>
                            { "Keep a physical copy of this phrase in a safe place." }
                        </p>
                        <div class={classes!("font-bold", "text-sm", "grid", "grid-cols-3", "gap-2")}>
                            { mnemonic.split_whitespace().enumerate().map(|(i, word)| {
                                html! {
                                    <div class={classes!("text-center", "text-sm", "font-bold", "flex", "gap-1")}>
                                        <p class={classes!("font-bold", "text-muted-foreground", "select-none", "pointer-events-none")}>
                                            { format!("{}. ", i + 1) }
                                        </p>
                                        <p class={classes!("text-muted-foreground")}>{ format!("{word}") }</p>
                                    </div>
                                }
                            }).collect::<Html>() }
                        </div>
                    </TabsContent>
                    <TabsContent value="hex-key" class={classes!("space-y-4")}>
                        <p class={classes!("font-bold", "text-muted-foreground", "select-none", "pointer-events-none")}>
                            { "Save this key in your password manager." }
                        </p>
                        <div class={classes!("flex", "gap-2" , "items-center")}>
                        <h3 class={classes!("flex-1", "font-medium", "leading-none", "text-wrap", "max-w-xs", "break-all", "mr-4")}>
                            { &*hex_key }
                        </h3>
                        <Button
                            onclick={copy_key}
                            r#type={shady_minions::ui::ButtonType::Button}
                            variant={shady_minions::ui::ButtonVariant::Outline}>
                            <crate::components::Copy size=6 class={classes!("text-muted-foreground")} />
                        </Button>
                        </div>
                    </TabsContent>
                </Tabs>
                <Button
                    r#type={shady_minions::ui::ButtonType::Button}
                    {onclick}
                    class={classes!("mt-4", "mr-4")}>
                    { "Save" }
                </Button>
                <Button
                    r#type={shady_minions::ui::ButtonType::Button}
                    variant={shady_minions::ui::ButtonVariant::Outline}
                    onclick={generate_new_key}
                    class={classes!("mt-4", "mr-4")}>
                    { "Generate New" }
                </Button>
            </CardContent>
        </Card>
    }
}
