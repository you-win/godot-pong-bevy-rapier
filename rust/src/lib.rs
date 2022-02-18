use gdnative::prelude::*;

mod ecs;

fn init(handle: InitHandle) {
    handle.add_class::<ecs::EcsFactory>();
    handle.add_class::<ecs::Ecs>();
    handle.add_class::<ecs::RustSprite>();
}

godot_init!(init);
