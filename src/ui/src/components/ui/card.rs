use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    let classes = classes!(
        "rounded-lg",
        "border",
        "border-border",
        "bg-card",
        "text-card-foreground",
        "shadow-sm",
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

#[derive(Properties, PartialEq)]
pub struct CardHeaderProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(CardHeader)]
pub fn card_header(props: &CardHeaderProps) -> Html {
    let classes = classes!(
        "flex",
        "flex-col",
        "space-y-1.5",
        "p-6",
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

#[derive(Properties, PartialEq)]
pub struct CardTitleProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(CardTitle)]
pub fn card_title(props: &CardTitleProps) -> Html {
    let classes = classes!(
        "text-lg",
        "font-semibold",
        "leading-none",
        "tracking-tight",
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    html! {
        <h3 class={classes}>
            { for props.children.iter() }
        </h3>
    }
}

#[derive(Properties, PartialEq)]
pub struct CardDescriptionProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(CardDescription)]
pub fn card_description(props: &CardDescriptionProps) -> Html {
    let classes = classes!(
        "text-sm",
        "text-muted-foreground",
        props
            .class
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_default()
    );

    html! {
        <p class={classes}>
            { for props.children.iter() }
        </p>
    }
}

#[derive(Properties, PartialEq)]
pub struct CardContentProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(CardContent)]
pub fn card_content(props: &CardContentProps) -> Html {
    let classes = classes!(
        "p-6",
        "pt-0",
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

#[derive(Properties, PartialEq)]
pub struct CardFooterProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(CardFooter)]
pub fn card_footer(props: &CardFooterProps) -> Html {
    let classes = classes!(
        "flex",
        "items-center",
        "p-6",
        "pt-0",
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
