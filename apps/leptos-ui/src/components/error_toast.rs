use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct ErrorInfo {
    pub message: String,
    pub context: Option<String>,
    pub recovery: Option<String>,
}

impl ErrorInfo {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
            recovery: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_recovery(mut self, recovery: impl Into<String>) -> Self {
        self.recovery = Some(recovery.into());
        self
    }
}

#[component]
pub fn ErrorToast(
    error_signal: ReadSignal<Option<ErrorInfo>>,
    set_error: WriteSignal<Option<ErrorInfo>>,
) -> impl IntoView {
    view! {
        {move || {
            error_signal
                .get()
                .map(|error| {
                    view! {
                        <div class="fixed top-4 right-4 z-50 max-w-md animate-in slide-in-from-right" role="alert">
                            <div class="bg-white rounded-lg shadow-xl border-l-4 border-red-500 p-4">
                                <div class="flex items-start justify-between mb-3">
                                    <div class="flex items-center gap-2">
                                        <span class="text-2xl">{"⚠️"}</span>
                                        <h3 class="text-lg font-semibold text-slate-800">"Error"</h3>
                                    </div>
                                    <button
                                        class="text-slate-400 hover:text-slate-600 transition-colors p-1"
                                        on:click=move |_| set_error.set(None)
                                        aria-label="Close error notification"
                                    >
                                        <span class="text-xl">"✕"</span>
                                    </button>
                                </div>

                                <div>
                                    <p class="text-slate-700 mb-3">{error.message.clone()}</p>

                                    {error
                                        .context
                                        .as_ref()
                                        .map(|ctx| {
                                            view! {
                                                <details class="mb-3">
                                                    <summary class="cursor-pointer text-sm text-slate-600 hover:text-slate-800 font-medium">"Technical Details"</summary>
                                                    <p class="mt-2 text-xs text-slate-600 bg-slate-50 p-2 rounded">{ctx.clone()}</p>
                                                </details>
                                            }
                                        })}

                                    {error
                                        .recovery
                                        .as_ref()
                                        .map(|rec| {
                                            view! {
                                                <div class="bg-indigo-600/5 border border-indigo-600/20 rounded p-3 text-sm">
                                                    <strong class="text-indigo-600">"Suggestion: "</strong>
                                                    <span class="text-slate-700">{rec.clone()}</span>
                                                </div>
                                            }
                                        })}
                                </div>
                            </div>
                        </div>
                    }
                })
        }}
    }
}

// Helper function to show errors from command results
pub fn handle_command_error(
    error_str: String,
    set_error: WriteSignal<Option<ErrorInfo>>,
) {
    // Parse error string for context and recovery hints
    let mut error_info = ErrorInfo::new(extract_user_message(&error_str));

    // Check if error contains module context (e.g., "usecases::timer::start_timer_session")
    if error_str.contains("usecases::") || error_str.contains("infra::") {
        error_info.context = Some(error_str.clone());
    }

    // Add recovery suggestions based on error type
    if error_str.contains("TaskNotFound") {
        error_info.recovery =
            Some("Please select a valid task from the task list".to_string());
    } else if error_str.contains("TaskAlreadyCompleted") {
        error_info.recovery = Some(
            "This task is complete. Create a new task or reset the current one"
                .to_string(),
        );
    } else if error_str.contains("InvalidStateTransition") {
        error_info.recovery = Some("Please ensure the timer is in the correct state for this operation".to_string());
    } else if error_str.contains("Failed to start timer") {
        error_info.recovery = Some(
            "Make sure a task is selected before starting the timer"
                .to_string(),
        );
    }

    set_error.set(Some(error_info));
}

fn extract_user_message(error_str: &str) -> String {
    // Extract the most relevant part of the error for the user
    if let Some(idx) = error_str.rfind(" - ") {
        // Get the part after the last " - " which is usually the actual error
        let message = &error_str[idx + 3..];

        // Clean up technical prefixes
        message
            .replace("Failed to ", "Could not ")
            .replace("Error: ", "")
            .to_string()
    } else {
        // Fallback: clean up common technical terms
        error_str
            .replace("Error: ", "")
            .replace("Failed to ", "Could not ")
            .to_string()
    }
}
