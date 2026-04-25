use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[component]
pub fn App() -> impl IntoView {
    let (files, set_files) = signal(Vec::<String>::new());
    let (status_map, set_status_map) = signal(HashMap::<String, bool>::new());
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
                            set_files.set(list.clone());

                            let mut status_map_local = HashMap::new();

                            for file in list.iter() {
                                if file.contains("_latest.json") {
                                    let url = format!(
                                        "https://api.jdecnc.cloud/file/{}",
                                        file
                                    );

                                    if let Ok(resp) = Request::get(&url).send().await {
                                        if let Ok(data) = resp.json::<Vec<Value>>().await {
                                            let mut all_ok = true;

                                            for item in data.iter() {
                                                let instance_status = item
                                                    .get("instance_status")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("");

                                                let health_status = item
                                                    .get("health_status")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("");

                                                if instance_status != "RUNNING"
                                                    || health_status != "passed"
                                                {
                                                    all_ok = false;
                                                    break;
                                                }
                                            }

                                            status_map_local.insert(file.clone(), all_ok);
                                        }
                                    }
                                }
                            }

                            set_status_map.set(status_map_local);
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

            <button
                on:click=load_files
                style="padding: 10px 16px; border-radius: 6px; cursor: pointer;"
            >
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
                let mut grouped: HashMap<String, HashSet<String>> = HashMap::new();

                for file in files.get() {
                    let parts: Vec<&str> = file.split('_').collect();

                    if parts.len() >= 2 {
                        let customer = parts[0].to_string();
                        let env = parts[1].to_string();

                        grouped.entry(customer).or_default().insert(env);
                    }
                }

                view! {
                    <div style="margin-top:20px;">
                        {grouped.into_iter().map(|(customer, envs)| {

                            let customer_name = customer.clone();

                            let mut env_list: Vec<String> = envs.into_iter().collect();
                            env_list.sort();

                            view! {
                                <div style="margin-bottom:15px; border:1px solid #ddd; padding:10px; border-radius:8px;">
                                    <h3>{customer}</h3>

                                    <ul>
                                        {env_list.into_iter().map(move |env| {

                                            let key = format!("{}_{}_latest.json", customer_name, env);

                                            let status = status_map
                                                .get()
                                                .get(&key)
                                                .cloned()
                                                .unwrap_or(false);

                                            view! {
                                                <li>
                                                    {env}
                                                    " → "
                                                    <span style=format!(
                                                        "color:{}; font-weight:bold;",
                                                        if status { "green" } else { "red" }
                                                    )>
                                                        {if status { "Healthy" } else { "Issues" }}
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