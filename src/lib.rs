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

// ✅ FIXED: unsafe extern
unsafe extern "C" {
    fn updateChart(healthy: usize, issue: usize, persistent: usize);
}

#[component]
pub fn App() -> impl IntoView {
    let (data, set_data) = signal(Vec::<SummaryItem>::new());
    let (loading, set_loading) = signal(true);

    let fetch_data = move || {
        spawn_local(async move {
            if let Ok(resp) = Request::get("https://api.jdecnc.cloud/summary")
                .send()
                .await
            {
                if let Ok(list) = resp.json::<Vec<SummaryItem>>().await {
                    set_data.set(list);
                }
            }

            set_loading.set(false);
        });
    };

    // Auto refresh
    Effect::new(move |_| {
        fetch_data();

        Interval::new(300_000, move || {
            fetch_data();
        });
    });

    // Update chart
    Effect::new(move |_| {
        let d = data.get();

        if d.is_empty() {
            return;
        }

        let healthy = d.iter().filter(|i| i.status == "Healthy").count();
        let issue = d.iter().filter(|i| i.status == "Issue").count();
        let persistent = d.iter().filter(|i| i.status == "Persistent").count();

        unsafe {
            updateChart(healthy, issue, persistent);
        }
    });

    view! {
        <main style="padding:20px; font-family:Arial;">
            <h1>"SMC Dashboard"</h1>

            <Show when=move || loading.get() fallback=|| view! {} >
                <p>"Loading..."</p>
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
                        margin-top:20px;
                    ">

                        {grouped.into_iter().map(|(customer, items)| {
                            view! {
                                <div style="background:white; padding:16px;">
                                    <h3>{customer}</h3>

                                    <ul>
                                        {items.into_iter().map(|i| {
                                            let color = match i.status.as_str() {
                                                "Healthy" => "green",
                                                "Issue" => "orange",
                                                "Persistent" => "red",
                                                _ => "gray",
                                            };

                                            view! {
                                                <li>
                                                    {i.env} " → "
                                                    <span style=format!("color:{};", color)>
                                                        {i.status}
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