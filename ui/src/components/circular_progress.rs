use domain::Phase;
use leptos::prelude::*;

#[component]
pub fn CircularProgress(
    #[prop(into)] progress: Signal<f64>,
    #[prop(into)] phase: Signal<Phase>,
) -> impl IntoView {
    let stroke_dasharray = 2.0 * std::f64::consts::PI * 90.0;

    view! {
        <div class="relative">
            <svg class="progress-ring" width="200" height="200">
                <circle
                    class="stroke-gray-200 dark:stroke-gray-600"
                    stroke-width="8"
                    fill="transparent"
                    r="90"
                    cx="100"
                    cy="100"
                />
                <circle
                    class="progress-ring-circle"
                    stroke={move || match phase.get() {
                        Phase::Work => "#ef4444",
                        Phase::ShortBreak => "#22c55e",
                        Phase::LongBreak => "#3b82f6"
                    }}
                    stroke-width="8"
                    fill="transparent"
                    r="90"
                    cx="100"
                    cy="100"
                    style:stroke-dasharray=format!("{stroke_dasharray}")
                    style:stroke-dashoffset={move || {
                        let offset = stroke_dasharray * (1.0 - progress.get() / 100.0);
                        format!("{offset}")
                    }}
                />
            </svg>
        </div>
    }
}
