use crate::components::ui::button::{Button, ButtonVariant};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    #[prop_or_default]
    pub on_add_task: Option<Callback<()>>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    let on_add_click = {
        let on_add_task = props.on_add_task.clone();
        Callback::from(move |_| {
            if let Some(callback) = &on_add_task {
                callback.emit(());
            }
        })
    };

    html! {
        <header class="border-b border-border bg-background">
            <div class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <h1 class="text-2xl font-bold text-foreground">
                            {"Luce"}
                        </h1>
                        <p class="text-sm text-muted-foreground">
                            {"Parallel Task Manager"}
                        </p>
                    </div>

                    <Button
                        variant={ButtonVariant::Primary}
                        onclick={on_add_click}
                    >
                        {"+ Add Task"}
                    </Button>
                </div>
            </div>
        </header>
    }
}
