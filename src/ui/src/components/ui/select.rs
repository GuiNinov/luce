use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SelectProps {
    pub children: Children,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub required: bool,
    #[prop_or_default]
    pub class: Option<AttrValue>,
    #[prop_or_default]
    pub onchange: Option<Callback<Event>>,
}

#[function_component(Select)]
pub fn select(props: &SelectProps) -> Html {
    let classes = classes!(
        "flex",
        "h-10",
        "w-full",
        "items-center",
        "justify-between",
        "rounded-md",
        "border",
        "border-input",
        "bg-background",
        "px-3",
        "py-2",
        "text-sm",
        "ring-offset-background",
        "placeholder:text-muted-foreground",
        "focus:outline-none",
        "focus:ring-2",
        "focus:ring-ring",
        "focus:ring-offset-2",
        "disabled:cursor-not-allowed",
        "disabled:opacity-50",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    let onchange = props.onchange.clone();

    html! {
        <select
            class={classes}
            value={props.value.clone()}
            disabled={props.disabled}
            required={props.required}
            onchange={move |e| {
                if let Some(callback) = &onchange {
                    callback.emit(e);
                }
            }}
        >
            { for props.children.iter() }
        </select>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectOptionProps {
    pub children: Children,
    pub value: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
}

#[function_component(SelectOption)]
pub fn select_option(props: &SelectOptionProps) -> Html {
    html! {
        <option 
            value={props.value.clone()}
            disabled={props.disabled}
        >
            { for props.children.iter() }
        </option>
    }
}