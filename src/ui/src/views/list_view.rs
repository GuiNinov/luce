use crate::components::task::task_card::TaskCard;
use luce_shared::task::Task;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ListViewProps {
    pub tasks: Vec<Task>,
}

#[function_component(ListView)]
pub fn list_view(props: &ListViewProps) -> Html {
    let selected_task = use_state(|| None::<usize>);

    if props.tasks.is_empty() {
        html! {
            <div class="flex flex-col items-center justify-center py-12 text-center">
                <div class="text-4xl mb-4">{"📝"}</div>
                <h3 class="text-lg font-medium text-foreground mb-2">
                    {"No tasks yet"}
                </h3>
                <p class="text-sm text-muted-foreground max-w-md">
                    {"Get started by creating your first task. Click the 'Add Task' button above to begin organizing your work."}
                </p>
            </div>
        }
    } else {
        html! {
            <div class="space-y-4">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-foreground">
                        {"Tasks"}
                    </h2>
                    <span class="text-sm text-muted-foreground">
                        {format!("{} task{}", props.tasks.len(), if props.tasks.len() == 1 { "" } else { "s" })}
                    </span>
                </div>

                <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {for props.tasks.iter().enumerate().map(|(index, task)| {
                        let selected = *selected_task == Some(index);
                        let task_clone = task.clone();
                        let selected_task_clone = selected_task.clone();

                        let on_select = Callback::from(move |_| {
                            if selected {
                                selected_task_clone.set(None);
                            } else {
                                selected_task_clone.set(Some(index));
                            }
                        });

                        html! {
                            <TaskCard
                                key={task.id.to_string()}
                                task={task_clone}
                                selected={selected}
                                on_select={on_select}
                            />
                        }
                    })}
                </div>
            </div>
        }
    }
}
