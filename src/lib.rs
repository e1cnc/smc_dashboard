use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

#[component]
pub fn App() -> impl IntoView {
    let (files, set_files) = signal(Vec::<String>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let load_files = move |_| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let result = Request::get("https://api.jdecnc.cloud/list")
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let json = resp.json::<Vec<String>>().await;
                    match json {
                        Ok(list) => {
                            set_files.set(list);
                            set_loading.set(false);
                        }
                        Err(e) => {
                            set_error.set(Some(format!("JSON error: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    set_error.set(Some(format!("Request error: {e}")));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <main style="padding: 24px; font-family: Arial;">
            <h1>"SMC Dashboard"</h1>

            <button on:click=load_files>
                "Load files"
            </button>

            <Show when=move || loading.get() fallback=|| view! {} >
                <p>"Loading..."</p>
            </Show>

            <Show when=move || error.get().is_some() fallback=|| view! {} >
                <p style="color:red;">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>

            {move || {
                let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

                for file in files.get() {
                    let parts: Vec<&str> = file.split('_').collect();

                    if parts.len() >= 2 {
                        let customer = parts[0].to_string();
                        let env = parts[1].to_string();

                        grouped.entry(customer).or_default().push(env);
                    }
                }

                view! {
                    <div style="margin-top:20px;">
                        {grouped.into_iter().map(|(customer, envs)| {
                            view! {
                                <div style="margin-bottom:15px; border:1px solid #ddd; padding:10px;">
                                    <h3>{customer}</h3>

                                    <ul>
                                        {envs.into_iter().map(|env| {
                                            view! {
                                                <li>
                                                    {env}
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }
            }}
        </main>
    }
}