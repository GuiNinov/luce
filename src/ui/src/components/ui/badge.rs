use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
}

impl Default for BadgeVariant {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Properties, PartialEq)]
pub struct BadgeProps {
    pub children: Children,
    #[prop_or_default]
    pub variant: BadgeVariant,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(Badge)]
pub fn badge(props: &BadgeProps) -> Html {
    let base_classes = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";

    let variant_classes = match props.variant {
        BadgeVariant::Default => {
            "border-transparent bg-primary text-primary-foreground hover:bg-primary/80"
        }
        BadgeVariant::Secondary => {
            "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
        }
        BadgeVariant::Destructive => {
            "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80"
        }
        BadgeVariant::Outline => "text-foreground",
    };

    let classes = classes!(
        base_classes,
        variant_classes,
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    html! {
        <div class={classes}>
            { for props.children.iter() }
        </div>
    }
}
