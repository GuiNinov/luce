use yew::prelude::*;
use luce_shared::task::Task;

#[derive(Properties, PartialEq)]
pub struct GraphViewProps {
    pub tasks: Vec<Task>,
}

#[function_component(GraphView)]
pub fn graph_view(props: &GraphViewProps) -> Html {
    if props.tasks.is_empty() {
        html! {
            <div class="flex flex-col items-center justify-center py-12 text-center">
                <div class="text-4xl mb-4">{"🕸️"}</div>
                <h3 class="text-lg font-medium text-foreground mb-2">
                    {"No tasks to visualize"}
                </h3>
                <p class="text-sm text-muted-foreground max-w-md">
                    {"The graph view will show task dependencies and relationships once you have created some tasks."}
                </p>
            </div>
        }
    } else {
        html! {
            <div class="space-y-4">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-foreground">
                        {"Task Graph"}
                    </h2>
                    <span class="text-sm text-muted-foreground">
                        {format!("{} node{}", props.tasks.len(), if props.tasks.len() == 1 { "" } else { "s" })}
                    </span>
                </div>

                // Placeholder for graph visualization
                <div class="border border-border rounded-lg p-8 bg-card min-h-[400px] flex items-center justify-center">
                    <div class="text-center">
                        <div class="text-2xl mb-4">{"🚧"}</div>
                        <h3 class="text-lg font-medium text-foreground mb-2">
                            {"Graph view coming soon"}
                        </h3>
                        <p class="text-sm text-muted-foreground max-w-md">
                            {"SVG-based graph visualization will be implemented here to show task relationships and dependencies."}
                        </p>
                        
                        // Simple task list for now
                        <div class="mt-6 text-left">
                            <h4 class="text-sm font-medium mb-2">{"Current Tasks:"}</h4>
                            <ul class="text-sm text-muted-foreground space-y-1">
                                {for props.tasks.iter().take(5).map(|task| {
                                    html! {
                                        <li class="flex items-center space-x-2">
                                            <span class="w-2 h-2 bg-primary rounded-full"></span>
                                            <span>{&task.title}</span>
                                        </li>
                                    }
                                })}
                                {if props.tasks.len() > 5 {
                                    html! {
                                        <li class="text-xs italic">
                                            {format!("... and {} more", props.tasks.len() - 5)}
                                        </li>
                                    }
                                } else {
                                    html! {}
                                }}
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}