use yew::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use luce_shared::task::{Task, TaskPriority};

use crate::components::ui::dialog::{Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::input::Input;
use crate::components::ui::textarea::Textarea;
use crate::components::ui::select::{Select, SelectOption};

#[derive(Properties, PartialEq)]
pub struct AddTaskModalProps {
    pub open: bool,
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
    #[prop_or_default]
    pub oncreate: Option<Callback<Task>>,
}

#[function_component(AddTaskModal)]
pub fn add_task_modal(props: &AddTaskModalProps) -> Html {
    let title = use_state(|| String::new());
    let description = use_state(|| String::new());
    let priority = use_state(|| TaskPriority::Normal);

    // Reset form when modal closes
    use_effect_with(props.open, {
        let title = title.clone();
        let description = description.clone();
        let priority = priority.clone();
        
        move |&is_open| {
            if !is_open {
                title.set(String::new());
                description.set(String::new());
                priority.set(TaskPriority::Normal);
            }
        }
    });

    let on_title_input = {
        let title = title.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            title.set(input.value());
        })
    };

    let on_description_input = {
        let description = description.clone();
        Callback::from(move |e: InputEvent| {
            let textarea = e.target().unwrap().dyn_into::<web_sys::HtmlTextAreaElement>().unwrap();
            description.set(textarea.value());
        })
    };

    let on_priority_change = {
        let priority = priority.clone();
        Callback::from(move |e: Event| {
            let select = e.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let priority_value = match select.value().as_str() {
                "low" => TaskPriority::Low,
                "normal" => TaskPriority::Normal,
                "high" => TaskPriority::High,
                "critical" => TaskPriority::Critical,
                _ => TaskPriority::Normal,
            };
            priority.set(priority_value);
        })
    };

    let on_cancel = {
        let onclose = props.onclose.clone();
        Callback::from(move |_| {
            if let Some(callback) = &onclose {
                callback.emit(());
            }
        })
    };

    let on_create = {
        let title = title.clone();
        let description = description.clone();
        let priority = priority.clone();
        let oncreate = props.oncreate.clone();
        
        Callback::from(move |_| {
            if !title.trim().is_empty() {
                let task = if description.trim().is_empty() {
                    Task::new(title.trim().to_string())
                        .with_priority((*priority).clone())
                } else {
                    Task::new(title.trim().to_string())
                        .with_description(description.trim().to_string())
                        .with_priority((*priority).clone())
                };
                
                if let Some(callback) = &oncreate {
                    callback.emit(task);
                }
            }
        })
    };

    let can_create = !title.trim().is_empty();
    let priority_value = match *priority {
        TaskPriority::Low => "low",
        TaskPriority::Normal => "normal", 
        TaskPriority::High => "high",
        TaskPriority::Critical => "critical",
    };

    html! {
        <Dialog open={props.open} onclose={props.onclose.clone()}>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>{"Create New Task"}</DialogTitle>
                    <DialogDescription>
                        {"Add a new task to your workflow. Tasks will automatically be set to Ready status if they have no dependencies."}
                    </DialogDescription>
                </DialogHeader>

                <div class="grid gap-4 py-4">
                    <div class="grid gap-2">
                        <label class="text-sm font-medium text-foreground" for="task-title">
                            {"Title *"}
                        </label>
                        <Input
                            placeholder="Enter task title..."
                            value={(*title).clone()}
                            required=true
                            oninput={on_title_input}
                        />
                    </div>

                    <div class="grid gap-2">
                        <label class="text-sm font-medium text-foreground" for="task-description">
                            {"Description"}
                        </label>
                        <Textarea
                            placeholder="Enter task description (optional)..."
                            value={(*description).clone()}
                            rows="3"
                            oninput={on_description_input}
                        />
                    </div>

                    <div class="grid gap-2">
                        <label class="text-sm font-medium text-foreground" for="task-priority">
                            {"Priority"}
                        </label>
                        <Select
                            value={priority_value}
                            onchange={on_priority_change}
                        >
                            <SelectOption value="low">{"Low"}</SelectOption>
                            <SelectOption value="normal">{"Normal"}</SelectOption>
                            <SelectOption value="high">{"High"}</SelectOption>
                            <SelectOption value="critical">{"Critical"}</SelectOption>
                        </Select>
                    </div>
                </div>

                <DialogFooter>
                    <Button 
                        variant={ButtonVariant::Outline}
                        onclick={on_cancel}
                    >
                        {"Cancel"}
                    </Button>
                    <Button 
                        variant={ButtonVariant::Primary}
                        disabled={!can_create}
                        onclick={on_create}
                    >
                        {"Create Task"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    }
}