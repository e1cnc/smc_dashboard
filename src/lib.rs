use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

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
//body part
    view! {
        <main style="padding: 24px; font-family: Arial, sans-serif;">
            <h1>"SMC Dashboard"</h1>

            <button
                on:click=load_files
                style="padding: 10px 16px; border: none; border-radius: 8px; background: #2563eb; color: white; cursor: pointer;"
            >
                "Load files"
            </button>

            <Show when=move || loading.get() fallback=|| view! {}>
                <p>"Loading..."</p>
            </Show>

            <Show when=move || error.get().is_some() fallback=|| view! {}>
                <p style="color: red;">
                    {move || error.get().unwrap_or_default()}
                </p>
            </Show>

            <ul>
                {move || {
                    files
                        .get()
                        .into_iter()
                        .map(|file| view! { <li>{file}</li> })
                        .collect_view()
                }}
            </ul>
        </main>
    }
}