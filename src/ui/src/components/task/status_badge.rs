use crate::components::ui::badge::{Badge, BadgeVariant};
use luce_shared::task::TaskStatus;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBadgeProps {
    pub status: TaskStatus,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(StatusBadge)]
pub fn status_badge(props: &StatusBadgeProps) -> Html {
    let (variant, text, color_class) = match props.status {
        TaskStatus::Pending => (
            BadgeVariant::Secondary,
            "Pending",
            "bg-muted text-muted-foreground",
        ),
        TaskStatus::Ready => (
            BadgeVariant::Default,
            "Ready",
            "bg-green-500 text-white hover:bg-green-600",
        ),
        TaskStatus::InProgress => (
            BadgeVariant::Default,
            "In Progress",
            "bg-blue-500 text-white hover:bg-blue-600",
        ),
        TaskStatus::Completed => (
            BadgeVariant::Default,
            "Completed",
            "bg-green-600 text-white hover:bg-green-700",
        ),
        TaskStatus::Failed => (
            BadgeVariant::Destructive,
            "Failed",
            "bg-red-500 text-white hover:bg-red-600",
        ),
        TaskStatus::Blocked => (
            BadgeVariant::Default,
            "Blocked",
            "bg-orange-500 text-white hover:bg-orange-600",
        ),
    };

    html! {
        <Badge
            variant={variant}
            class={if let Some(ref additional) = props.class {
                classes!(color_class, additional.to_string())
            } else {
                classes!(color_class)
            }}
        >
            {text}
        </Badge>
    }
}
