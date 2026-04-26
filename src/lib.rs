use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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

    let (seconds_left, set_seconds_left) = signal(300);
    let (last_updated, set_last_updated) = signal(String::from("Never"));

    let fetch_data = move || {
        set_loading.set(true);

        spawn_local(async move {
            if let Ok(resp) = Request::get("https://api.jdecnc.cloud/summary")
                .send()
                .await
            {
                if let Ok(list) = resp.json::<Vec<SummaryItem>>().await {
                    set_data.set(list);
                }
            }

            set_seconds_left.set(300);

            let now = js_sys::Date::new_0();
            let time_str = now.to_locale_time_string("en-GB");
            set_last_updated.set(time_str.into());

            set_loading.set(false);
        });
    };

    // 🔁 Auto refresh
    {
        let fetch_clone = fetch_data.clone();

        let interval_handle: Rc<RefCell<Option<Interval>>> = Rc::new(RefCell::new(None));
        let interval_handle_clone = interval_handle.clone();

        Effect::new(move |_| {
            fetch_clone();

            *interval_handle_clone.borrow_mut() = Some(
                Interval::new(1000, move || {
                    let current = seconds_left.get();

                    if current <= 1 {
                        fetch_clone();
                    } else {
                        set_seconds_left.set(current - 1);
                    }
                })
            );
        });
    }

    view! {
        <main style="
            padding: 20px;
            font-family: Arial;
            background:#f5f7fa;
            max-width: 1400px;
            margin: 0 auto;
        ">
            <h1>"SMC Dashboard"</h1>

            // Status bar
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

                grouped_vec.sort_by(|a, b| {
                    let score = |items: &Vec<SummaryItem>| {
                        items.iter().filter(|i| i.status != "Healthy").count()
                    };
                    score(&b.1).cmp(&score(&a.1))
                });

                view! {
                    <>
                        // Summary
                        <div style="
                            margin: 20px 0;
                            font-weight: bold;
                            display:flex;
                            flex-wrap:wrap;
                            gap:15px;
                        ">
                            <span>"Total: " {total}</span>
                            <span style="color:green;">"Healthy: " {healthy}</span>
                            <span style="color:orange;">"Issues: " {issue}</span>
                            <span style="color:red;">"Persistent: " {persistent}</span>
                        </div>

                        // ✅ RESPONSIVE GRID FIX
                        <div style="
                            display:grid;
                            grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
                            gap:20px;
                            width:100%;
                        ">

                            {grouped_vec.into_iter().map(|(customer, items)| {

                                let mut envs = items.clone();
                                envs.sort_by(|a, b| a.env.cmp(&b.env));

                                view! {
                                    <div style="
                                        background:white;
                                        padding:16px;
                                        border-radius:10px;
                                        box-shadow:0 2px 8px rgba(0,0,0,0.08);
                                        width:100%;
                                        box-sizing:border-box;
                                        min-height:150px;
                                    ">
                                        <h3 style="margin-bottom:10px;">{customer}</h3>

                                        <ul style="list-style:none; padding:0; margin:0;">
                                            {envs.into_iter().map(|item| {

                                                let key = (item.customer.clone(), item.env.clone());
                                                let key_click = key.clone();
                                                let key_show = key.clone();

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
                                                                align-items:center;
                                                                cursor:pointer;
                                                                gap:10px;
                                                            "
                                                            on:click=move |_| {
                                                                if expanded.get() == Some(key_click.clone()) {
                                                                    set_expanded.set(None);
                                                                } else {
                                                                    set_expanded.set(Some(key_click.clone()));
                                                                }
                                                            }
                                                        >
                                                            <span style="flex:1;">{item.env.clone()}</span>

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
                                                            when=move || expanded.get() == Some(key_show.clone())
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
                    </>
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