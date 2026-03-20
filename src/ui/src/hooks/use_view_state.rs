use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ViewType {
    Graph,
    List,
}

impl Default for ViewType {
    fn default() -> Self {
        Self::List
    }
}

#[hook]
pub fn use_view_state() -> (ViewType, Callback<ViewType>) {
    let view_state = use_state(|| ViewType::default());
    
    let set_view = {
        let view_state = view_state.clone();
        Callback::from(move |new_view: ViewType| {
            view_state.set(new_view);
        })
    };

    ((*view_state).clone(), set_view)
}