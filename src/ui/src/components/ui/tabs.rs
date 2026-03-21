use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TabsProps {
    pub children: Children,
    #[prop_or_default]
    pub default_value: Option<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<AttrValue>>,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(Tabs)]
pub fn tabs(props: &TabsProps) -> Html {
    let classes = classes!(
        "w-full",
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    html! {
        <div class={classes} data-orientation="horizontal" role="tablist">
            { for props.children.iter() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TabsListProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(TabsList)]
pub fn tabs_list(props: &TabsListProps) -> Html {
    let classes = classes!(
        "inline-flex",
        "h-10",
        "items-center",
        "justify-center",
        "rounded-md",
        "bg-muted",
        "p-1",
        "text-muted-foreground",
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    html! {
        <div class={classes} role="tablist">
            { for props.children.iter() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TabsTriggerProps {
    pub children: Children,
    pub value: AttrValue,
    #[prop_or_default]
    pub active: bool,
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(TabsTrigger)]
pub fn tabs_trigger(props: &TabsTriggerProps) -> Html {
    let base_classes = "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50";

    let state_classes = if props.active {
        "bg-background text-foreground shadow-sm"
    } else {
        "hover:bg-background/50 hover:text-foreground"
    };

    let classes = classes!(
        base_classes,
        state_classes,
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    let onclick = props.onclick.clone();

    html! {
        <button
            class={classes}
            role="tab"
            aria-selected={props.active.to_string()}
            data-value={&props.value}
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

#[derive(Properties, PartialEq)]
pub struct TabsContentProps {
    pub children: Children,
    pub value: AttrValue,
    #[prop_or_default]
    pub active: bool,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(TabsContent)]
pub fn tabs_content(props: &TabsContentProps) -> Html {
    let classes = classes!(
        "mt-2",
        "ring-offset-background",
        "focus-visible:outline-none",
        "focus-visible:ring-2",
        "focus-visible:ring-ring",
        "focus-visible:ring-offset-2",
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    if props.active {
        html! {
            <div
                class={classes}
                role="tabpanel"
                data-value={&props.value}
            >
                { for props.children.iter() }
            </div>
        }
    } else {
        html! {}
    }
}
