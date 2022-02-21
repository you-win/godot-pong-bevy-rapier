use gdnative::prelude::*;

mod ecs;

fn init(handle: InitHandle) {
    handle.add_class::<ecs::EcsFactory>();
    handle.add_class::<ecs::Ecs>();
}

godot_init!(init);
