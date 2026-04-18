use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <main style="min-height: 100vh; display: grid; place-items: center; font-family: Arial, sans-serif; background: #f8fafc;">
            <section style="background: white; padding: 24px; border-radius: 16px; box-shadow: 0 4px 20px rgba(0,0,0,0.08); max-width: 520px; width: 100%;">
                <h1 style="margin: 0 0 12px 0; font-size: 2rem;">"SMC Health Monitoring Dashboard"</h1>
                <p style="margin: 0 0 16px 0; color: #475569;">
                    "Your first Leptos app is running."
                </p>

                <button
                    on:click=move |_| set_count.update(|n| *n += 1)
                    style="border: none; background: #2563eb; color: white; padding: 10px 16px; border-radius: 10px; cursor: pointer; font-weight: 700;"
                >
                    "Clicked " {move || count.get()} " times"
                </button>
            </section>
        </main>
    }
}