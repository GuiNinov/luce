use yew::prelude::*;

use crate::components::layout::header::Header;
use crate::components::ui::tabs::{Tabs, TabsList, TabsTrigger, TabsContent};
use crate::hooks::use_view_state::{use_view_state, ViewType};
use crate::hooks::use_tasks::use_tasks;
use crate::views::list_view::ListView;
use crate::views::graph_view::GraphView;
use crate::components::task::add_task_modal::AddTaskModal;

#[function_component(App)]
pub fn app() -> Html {
    let (current_view, set_view) = use_view_state();
    let (tasks, add_task) = use_tasks();
    let show_add_modal = use_state(|| false);

    let on_add_task_click = {
        let show_add_modal = show_add_modal.clone();
        Callback::from(move |_| {
            show_add_modal.set(true);
        })
    };

    let on_modal_close = {
        let show_add_modal = show_add_modal.clone();
        Callback::from(move |_| {
            show_add_modal.set(false);
        })
    };

    let on_task_created = {
        let add_task = add_task.clone();
        let show_add_modal = show_add_modal.clone();
        Callback::from(move |task| {
            add_task.emit(task);
            show_add_modal.set(false);
        })
    };

    let on_graph_tab_click = {
        let set_view = set_view.clone();
        Callback::from(move |_| {
            set_view.emit(ViewType::Graph);
        })
    };

    let on_list_tab_click = {
        let set_view = set_view.clone();
        Callback::from(move |_| {
            set_view.emit(ViewType::List);
        })
    };

    html! {
        <div class="min-h-screen bg-background text-foreground">
            <Header on_add_task={on_add_task_click} />
            
            <main class="container mx-auto px-4 py-6">
                <Tabs class="w-full">
                    <TabsList class="grid w-full grid-cols-2 mb-6">
                        <TabsTrigger 
                            value="list" 
                            active={current_view == ViewType::List}
                            onclick={on_list_tab_click}
                        >
                            {"📋 List View"}
                        </TabsTrigger>
                        <TabsTrigger 
                            value="graph" 
                            active={current_view == ViewType::Graph}
                            onclick={on_graph_tab_click}
                        >
                            {"🕸️ Graph View"}
                        </TabsTrigger>
                    </TabsList>

                    <TabsContent 
                        value="list" 
                        active={current_view == ViewType::List}
                    >
                        <ListView tasks={tasks.clone()} />
                    </TabsContent>

                    <TabsContent 
                        value="graph" 
                        active={current_view == ViewType::Graph}
                    >
                        <GraphView tasks={tasks.clone()} />
                    </TabsContent>
                </Tabs>
            </main>

            <AddTaskModal
                open={*show_add_modal}
                onclose={on_modal_close}
                oncreate={on_task_created}
            />
        </div>
    }
}