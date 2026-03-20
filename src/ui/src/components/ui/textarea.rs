use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TextareaProps {
    #[prop_or_default]
    pub placeholder: Option<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub required: bool,
    #[prop_or_default]
    pub rows: Option<AttrValue>,
    #[prop_or_default]
    pub class: Option<AttrValue>,
    #[prop_or_default]
    pub oninput: Option<Callback<InputEvent>>,
    #[prop_or_default]
    pub onchange: Option<Callback<Event>>,
}

#[function_component(Textarea)]
pub fn textarea(props: &TextareaProps) -> Html {
    let classes = classes!(
        "flex",
        "min-h-[80px]",
        "w-full",
        "rounded-md",
        "border",
        "border-input",
        "bg-background",
        "px-3",
        "py-2",
        "text-sm",
        "ring-offset-background",
        "placeholder:text-muted-foreground",
        "focus-visible:outline-none",
        "focus-visible:ring-2",
        "focus-visible:ring-ring",
        "focus-visible:ring-offset-2",
        "disabled:cursor-not-allowed",
        "disabled:opacity-50",
        props.class.as_ref().map(|c| c.to_string()).unwrap_or_default()
    );

    let oninput = props.oninput.clone();
    let onchange = props.onchange.clone();
    let rows = props.rows.clone().unwrap_or_else(|| "3".into());

    html! {
        <textarea
            class={classes}
            placeholder={props.placeholder.clone()}
            value={props.value.clone()}
            disabled={props.disabled}
            required={props.required}
            rows={rows}
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
        >{props.value.clone().unwrap_or_default()}</textarea>
    }
}