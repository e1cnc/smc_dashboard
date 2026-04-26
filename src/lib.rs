use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct SummaryItem {
    customer: String,
    env: String,
    status: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DetailItem {
    instance_name: Option<String>,
    instance_status: Option<String>,
    health_status: Option<String>,
    details: Option<String>,
}

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(Vec::<SummaryItem>::new());
    let (loading, set_loading) = signal(true);
    let (expanded, set_expanded) = signal(None::<(String, String)>);

    let load_data = move || {
        spawn_local(async move {
            let result = Request::get("https://api.jdecnc.cloud/summary")
                .send()
                .await;

            if let Ok(resp) = result {
                if let Ok(list) = resp.json::<Vec<SummaryItem>>().await {
                    set_data.set(list);
                }
            }

            set_loading.set(false);
        });
    };

    // Auto refresh
    {
        let load_data_clone = load_data.clone();
        Effect::new(move |_| {
            load_data_clone();
            Interval::new(30000, move || {
                load_data_clone();
            });
        });
    }

    view! {
        <main style="padding: 24px; font-family: Arial; background:#f5f7fa;">
            <h1>"SMC Dashboard"</h1>

            <Show when=move || loading.get() fallback=|| view! {} >
                <p>"Loading dashboard..."</p>
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
                        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
                        gap:20px;
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
                                    <h3>{customer}</h3>

                                    <ul style="list-style:none; padding:0;">
                                        {envs.into_iter().map(|item| {

                                            // 🔥 FIX: clone key once and reuse safely
                                            let key = (item.customer.clone(), item.env.clone());
                                            let key_for_click = key.clone();
                                            let key_for_show = key.clone();

                                            let is_expanded = expanded.get() == Some(key.clone());

                                            let (label, color) = match item.status.as_str() {
                                                "Healthy" => ("Healthy", "#16a34a"),
                                                "Issue" => ("Issue", "#f59e0b"),
                                                "Persistent" => ("Persistent", "#dc2626"),
                                                _ => ("Unknown", "gray"),
                                            };

                                            view! {
                                                <li style="margin-bottom:8px;">
                                                    <div
                                                        style="
                                                            display:flex;
                                                            justify-content:space-between;
                                                            cursor:pointer;
                                                        "
                                                        on:click=move |_| {
                                                            if expanded.get() == Some(key_for_click.clone()) {
                                                                set_expanded.set(None);
                                                            } else {
                                                                set_expanded.set(Some(key_for_click.clone()));
                                                            }
                                                        }
                                                    >
                                                        <span>{item.env.clone()}</span>

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
                                                    </div>

                                                    <Show
                                                        when=move || expanded.get() == Some(key_for_show.clone())
                                                        fallback=|| view! {}
                                                    >
                                                        <DetailsPanel
                                                            customer=item.customer.clone()
                                                            env=item.env.clone()
                                                        />
                                                    </Show>
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

#[component]
fn DetailsPanel(customer: String, env: String) -> impl IntoView {
    let (details, set_details) = signal(Vec::<DetailItem>::new());
    let (loading, set_loading) = signal(true);

    spawn_local(async move {
        let url = format!("https://api.jdecnc.cloud/details/{}/{}", customer, env);

        if let Ok(resp) = Request::get(&url).send().await {
            if let Ok(data) = resp.json::<Vec<DetailItem>>().await {
                set_details.set(data);
            }
        }

        set_loading.set(false);
    });

    view! {
        <div style="
            margin-top:6px;
            padding:6px;
            background:#f9fafb;
            border-radius:6px;
            font-size:12px;
        ">
            <Show when=move || loading.get() fallback=|| view! {} >
                <p>"Loading details..."</p>
            </Show>

            <ul>
                {move || details.get().into_iter().map(|d| {
                    view! {
                        <li>
                            {d.instance_name.clone().unwrap_or("-".into())}
                            " → "
                            {d.instance_status.clone().unwrap_or("-".into())}
                            " / "
                            {d.health_status.clone().unwrap_or("-".into())}
                        </li>
                    }
                }).collect_view()}
            </ul>
        </div>
    }
}