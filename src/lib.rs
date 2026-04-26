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

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(Vec::<SummaryItem>::new());
    let (loading, set_loading) = signal(true);
    let (expanded, set_expanded) = signal(None::<(String, String)>);

    // 🆕 timer state (seconds)
    let (seconds_left, set_seconds_left) = signal(300);
    let (last_updated, set_last_updated) = signal(String::from("Never"));

    let fetch_data = move || {
        set_loading.set(true);

        spawn_local(async move {
            let result = Request::get("https://api.jdecnc.cloud/summary")
                .send()
                .await;

            if let Ok(resp) = result {
                if let Ok(list) = resp.json::<Vec<SummaryItem>>().await {
                    set_data.set(list);
                }
            }

            // reset timer
            set_seconds_left.set(300);

            // update timestamp
            let now = js_sys::Date::new_0();
            let time_str = now.to_locale_time_string("en-GB");
            set_last_updated.set(time_str.into());

            set_loading.set(false);
        });
    };

    // 🔁 countdown + auto refresh
    {
        let fetch_clone = fetch_data.clone();

        Effect::new(move |_| {
            fetch_clone();

            Interval::new(1000, move || {
                let current = seconds_left.get();

                if current <= 1 {
                    fetch_clone();
                } else {
                    set_seconds_left.set(current - 1);
                }
            });
        });
    }

    view! {
        <main style="padding: 24px; font-family: Arial; background:#f5f7fa;">
            <h1>"SMC Dashboard"</h1>

            // 🆕 status bar
            <div style="margin-bottom:10px; font-size:14px;">
                <b>"Last updated: "</b> {move || last_updated.get()}
                " | Refresh in: "
                <span style="color:#2563eb;">
                    {move || seconds_left.get()}
                </span>
                " sec"
            </div>

            <button on:click=move |_| fetch_data()>
                "Refresh Now"
            </button>

            <Show when=move || loading.get() fallback=|| view! {} >
                <p>"Loading dashboard..."</p>
            </Show>

            {move || {
                let total = data.get().len();
                let healthy = data.get().iter().filter(|i| i.status == "Healthy").count();
                let issue = data.get().iter().filter(|i| i.status == "Issue").count();
                let persistent = data.get().iter().filter(|i| i.status == "Persistent").count();

                let mut grouped: HashMap<String, Vec<SummaryItem>> = HashMap::new();

                for item in data.get() {
                    grouped.entry(item.customer.clone())
                        .or_default()
                        .push(item);
                }

                let mut grouped_vec: Vec<(String, Vec<SummaryItem>)> =
                    grouped.into_iter().collect();

                // 🔥 sort by severity
                grouped_vec.sort_by(|a, b| {
                    let score = |items: &Vec<SummaryItem>| {
                        items.iter().filter(|i| i.status != "Healthy").count()
                    };
                    score(&b.1).cmp(&score(&a.1))
                });

                view! {
                    <div style="margin-bottom:20px; font-weight:bold;">
                        <span>"Total: " {total} " "</span>
                        <span style="color:green;">"Healthy: " {healthy} " "</span>
                        <span style="color:orange;">"Issues: " {issue} " "</span>
                        <span style="color:red;">"Persistent: " {persistent}</span>
                    </div>

                    <div style="
                        display:grid;
                        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
                        gap:20px;
                    ">

                        {grouped_vec.into_iter().map(|(customer, items)| {

                            let mut envs = items.clone();
                            envs.sort_by(|a, b| a.env.cmp(&b.env));

                            view! {
                                <div style="background:white; padding:16px; border-radius:10px;">
                                    <h3>{customer}</h3>

                                    <ul style="list-style:none; padding:0;">
                                        {envs.into_iter().map(|item| {

                                            let key = (item.customer.clone(), item.env.clone());

                                            let (label, color) = match item.status.as_str() {
                                                "Healthy" => ("Healthy", "green"),
                                                "Issue" => ("Issue", "orange"),
                                                "Persistent" => ("Persistent", "red"),
                                                _ => ("Unknown", "gray"),
                                            };

                                            view! {
                                                <li style="margin-bottom:6px;">
                                                    <span>{item.env}</span>
                                                    " → "
                                                    <span style=format!("color:{};", color)>
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