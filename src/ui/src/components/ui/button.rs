use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    Sm,
    Default,
    Lg,
    Icon,
}

impl Default for ButtonSize {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub children: Children,
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub class: Option<AttrValue>,
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let base_classes = "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";

    let variant_classes = match props.variant {
        ButtonVariant::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Destructive => {
            "bg-destructive text-destructive-foreground hover:bg-destructive/90"
        }
        ButtonVariant::Outline => {
            "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
        }
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Link => "text-primary underline-offset-4 hover:underline",
    };

    let size_classes = match props.size {
        ButtonSize::Default => "h-10 px-4 py-2",
        ButtonSize::Sm => "h-9 rounded-md px-3",
        ButtonSize::Lg => "h-11 rounded-md px-8",
        ButtonSize::Icon => "h-10 w-10",
    };

    let additional_class = props
        .class
        .as_ref()
        .map(|c| c.to_string())
        .unwrap_or_default();
    let classes = if additional_class.is_empty() {
        classes!(base_classes, variant_classes, size_classes)
    } else {
        classes!(
            base_classes,
            variant_classes,
            size_classes,
            additional_class
        )
    };

    let onclick = props.onclick.clone();
    let disabled = props.disabled;

    html! {
        <button
            class={classes}
            disabled={disabled}
            onclick={move |e| {
                if let Some(callback) = &onclick {
                    callback.emit(e);
                }
            }}
        >
            { for props.children.iter() }
        </button>
    }
}
