use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct SummaryItem {
    customer: String,
    env: String,
    status: String,
}

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(Vec::<SummaryItem>::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);

    let load_data = move |_| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let result = Request::get("https://api.jdecnc.cloud/summary")
                .send()
                .await;

            match result {
                Ok(resp) => {
                    let json = resp.json::<Vec<SummaryItem>>().await;

                    match json {
                        Ok(list) => {
                            set_data.set(list);
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
        <main style="padding: 24px; font-family: Arial; background:#f5f7fa;">
            <h1>"SMC Dashboard"</h1>

            <button
                on:click=load_data
                style="
                    padding: 10px 16px;
                    border-radius: 6px;
                    border: none;
                    background: #2563eb;
                    color: white;
                    cursor: pointer;
                    margin-bottom: 16px;
                "
            >
                "Load Dashboard"
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
                let mut grouped: HashMap<String, Vec<SummaryItem>> = HashMap::new();

                for item in data.get() {
                    grouped.entry(item.customer.clone())
                        .or_default()
                        .push(item);
                }

                view! {
                    <div style="
                        display:grid;
                        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                        gap:16px;
                    ">

                        {grouped.into_iter().map(|(customer, items)| {

                            let mut envs = items.clone();
                            envs.sort_by(|a, b| a.env.cmp(&b.env));

                            view! {
                                <div style="
                                    background:white;
                                    padding:16px;
                                    border-radius:10px;
                                    box-shadow:0 2px 6px rgba(0,0,0,0.1);
                                ">
                                    <h3 style="margin-bottom:10px;">{customer}</h3>

                                    <ul style="list-style:none; padding:0;">
                                        {envs.into_iter().map(|item| {

                                            let (label, color) = match item.status.as_str() {
                                                "Healthy" => ("Healthy", "#16a34a"),
                                                "Issue" => ("New Issue", "#f59e0b"),
                                                "Persistent" => ("Persistent", "#dc2626"),
                                                _ => ("Unknown", "gray"),
                                            };

                                            view! {
                                                <li style="
                                                    display:flex;
                                                    justify-content:space-between;
                                                    padding:6px 0;
                                                ">
                                                    <span>{item.env}</span>

                                                    <span style=format!(
                                                        "padding:4px 8px;
                                                         border-radius:6px;
                                                         color:white;
                                                         font-size:12px;
                                                         background:{};",
                                                        color
                                                    )>
                                                        {label}
                                                    </span>
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