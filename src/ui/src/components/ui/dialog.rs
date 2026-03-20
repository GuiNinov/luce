use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub children: Children,
    #[prop_or_default]
    pub open: bool,
    #[prop_or_default]
    pub onclose: Option<Callback<()>>,
}

#[function_component(Dialog)]
pub fn dialog(props: &DialogProps) -> Html {
    let onclose = props.onclose.clone();
    
    let backdrop_onclick = {
        let onclose = onclose.clone();
        Callback::from(move |_| {
            if let Some(callback) = &onclose {
                callback.emit(());
            }
        })
    };

    if props.open {
        html! {
            <div class="fixed inset-0 z-50 flex items-center justify-center">
                // Backdrop
                <div 
                    class="fixed inset-0 bg-black/80" 
                    onclick={backdrop_onclick}
                ></div>
                
                // Dialog content
                <div class="relative z-50">
                    { for props.children.iter() }
                </div>
            </div>
        }
    } else {
        html! {}
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogContentProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(DialogContent)]
pub fn dialog_content(props: &DialogContentProps) -> Html {
    let classes = classes!(
        "fixed",
        "left-[50%]",
        "top-[50%]",
        "z-50",
        "grid",
        "w-full",
        "max-w-lg",
        "translate-x-[-50%]",
        "translate-y-[-50%]",
        "gap-4",
        "border",
        "border-border",
        "bg-background",
        "p-6",
        "shadow-lg",
        "duration-200",
        "rounded-lg",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    html! {
        <div class={classes} role="dialog">
            { for props.children.iter() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogHeaderProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(DialogHeader)]
pub fn dialog_header(props: &DialogHeaderProps) -> Html {
    let classes = classes!(
        "flex",
        "flex-col",
        "space-y-1.5",
        "text-center",
        "sm:text-left",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    html! {
        <div class={classes}>
            { for props.children.iter() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogTitleProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(DialogTitle)]
pub fn dialog_title(props: &DialogTitleProps) -> Html {
    let classes = classes!(
        "text-lg",
        "font-semibold",
        "leading-none",
        "tracking-tight",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    html! {
        <h2 class={classes}>
            { for props.children.iter() }
        </h2>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogDescriptionProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(DialogDescription)]
pub fn dialog_description(props: &DialogDescriptionProps) -> Html {
    let classes = classes!(
        "text-sm",
        "text-muted-foreground",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    html! {
        <p class={classes}>
            { for props.children.iter() }
        </p>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogFooterProps {
    pub children: Children,
    #[prop_or_default]
    pub class: Option<AttrValue>,
}

#[function_component(DialogFooter)]
pub fn dialog_footer(props: &DialogFooterProps) -> Html {
    let classes = classes!(
        "flex",
        "flex-col-reverse",
        "sm:flex-row",
        "sm:justify-end",
        "sm:space-x-2",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    html! {
        <div class={classes}>
            { for props.children.iter() }
        </div>
    }
}