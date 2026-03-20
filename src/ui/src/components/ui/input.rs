use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InputProps {
    #[prop_or_default]
    pub r#type: Option<AttrValue>,
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub required: bool,
    #[prop_or_default]
    pub class: Option<AttrValue>,
    #[prop_or_default]
    pub oninput: Option<Callback<InputEvent>>,
    #[prop_or_default]
    pub onchange: Option<Callback<Event>>,
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    let classes = classes!(
        "flex",
        "h-10",
        "w-full",
        "rounded-md",
        "border",
        "border-input",
        "bg-background",
        "px-3",
        "py-2",
        "text-sm",
        "ring-offset-background",
        "file:border-0",
        "file:bg-transparent",
        "file:text-sm",
        "file:font-medium",
        "placeholder:text-muted-foreground",
        "focus-visible:outline-none",
        "focus-visible:ring-2",
        "focus-visible:ring-ring",
        "focus-visible:ring-offset-2",
        "disabled:cursor-not-allowed",
        "disabled:opacity-50",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    let input_type = props.r#type.clone().unwrap_or_else(|| "text".into());
    let oninput = props.oninput.clone();
    let onchange = props.onchange.clone();

    html! {
        <input
            type={input_type}
            class={classes}
            placeholder={props.placeholder.clone()}
            value={props.value.clone()}
            disabled={props.disabled}
            required={props.required}
            oninput={move |e| {
                if let Some(callback) = &oninput {
                    callback.emit(e);
                }
            }}
            onchange={move |e| {
                if let Some(callback) = &onchange {
                    callback.emit(e);
                }
            }}
        />
    }
}