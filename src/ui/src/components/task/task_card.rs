use yew::prelude::*;
use luce_shared::task::{Task, TaskPriority};
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle, CardDescription};
use crate::components::task::status_badge::StatusBadge;

#[derive(Properties, PartialEq)]
pub struct TaskCardProps {
    pub task: Task,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub on_select: Option<Callback<()>>,
}

#[function_component(TaskCard)]
pub fn task_card(props: &TaskCardProps) -> Html {
    let onclick = {
        let on_select = props.on_select.clone();
        Callback::from(move |_| {
            if let Some(callback) = &on_select {
                callback.emit(());
            }
        })
    };

    let priority_indicator = match props.task.priority {
        TaskPriority::Critical => html! {
            <span class="text-red-500 font-bold" title="Critical Priority">{"🔴"}</span>
        },
        TaskPriority::High => html! {
            <span class="text-orange-500 font-medium" title="High Priority">{"🟠"}</span>
        },
        TaskPriority::Normal => html! {},
        TaskPriority::Low => html! {
            <span class="text-gray-400" title="Low Priority">{"⚪"}</span>
        },
    };

    let card_classes = if props.selected {
        "ring-2 ring-primary border-primary"
    } else {
        "hover:shadow-md transition-shadow cursor-pointer"
    };

    let creation_date = props.task.created_at.format("%b %d, %Y").to_string();

    html! {
        <Card 
            class={AttrValue::from(card_classes)}
        >
            <div onclick={onclick}>
                <CardHeader>
                    <div class="flex items-start justify-between">
                        <div class="flex items-center space-x-2">
                            {priority_indicator}
                            <CardTitle class="text-base">
                                {props.task.title.clone()}
                            </CardTitle>
                        </div>
                        <StatusBadge status={props.task.status.clone()} />
                    </div>
                    
                    {if let Some(ref description) = props.task.description {
                        html! {
                            <CardDescription>
                                {description.clone()}
                            </CardDescription>
                        }
                    } else {
                        html! {}
                    }}
                </CardHeader>

                <CardContent>
                    <div class="flex justify-between items-center text-xs text-muted-foreground">
                        <span>{"Created: "}{creation_date}</span>
                        <span class="text-xs bg-muted px-2 py-1 rounded">
                            {format!("ID: {}", &props.task.id.to_string()[..8])}
                        </span>
                    </div>
                </CardContent>
            </div>
        </Card>
    }
}