use std::collections::{vec_deque::Drain, HashMap, VecDeque};

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::RunOnce;
use gdnative::api::{GlobalConstants, ImageTexture, ProjectSettings, StreamTexture, VisualServer};
use gdnative::prelude::*;
use rapier2d::prelude::*;

#[derive(Default)]
struct Delta(f32);

#[derive(Component)]
struct Drawable {
    rid: Rid,
    transform: Transform2D,
}

const PADDLE_SPEED: f32 = 500.0;

#[derive(Component)]
enum Paddle {
    Left,
    Right,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum GodotInput {
    LeftUp,
    LeftDown,

    RightUp,
    RightDown,
}

struct InputQueue {
    queue: VecDeque<GodotInput>,
}

impl InputQueue {
    pub fn new() -> Self {
        let mut queue: VecDeque<GodotInput> = VecDeque::new();
        queue.make_contiguous();
        return InputQueue { queue: queue };
    }

    pub fn add(&mut self, data: GodotInput) {
        self.queue.push_back(data);
    }

    #[warn(dead_code)]
    pub fn read_single(&mut self) -> Option<GodotInput> {
        return self.queue.pop_front();
    }

    pub fn read_all(&mut self) -> Drain<'_, GodotInput> {
        return self.queue.drain(..);
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum Stages {
    Startup,
    Preupdate,
    Update,
    Postupdate,
}

#[derive(NativeClass)]
#[inherit(Reference)]
pub struct EcsFactory;

#[methods]
impl EcsFactory {
    fn new(_o: &Reference) -> Self {
        Self
    }

    #[export]
    fn new_ecs(&self, _o: &Reference) -> Instance<Ecs, Unique> {
        Ecs::new().emplace()
    }
}

#[derive(NativeClass)]
#[no_constructor]
#[inherit(Node2D)]
pub struct Ecs {
    schedule: Schedule,
    world: World,

    textures: Vec<Ref<Image>>,
}

#[methods]
impl Ecs {
    fn new() -> Self {
        let mut ecs = Ecs {
            schedule: Schedule::default(),
            world: World::default(),
            textures: Vec::new(),
        };

        ecs.world.insert_resource(InputQueue::new());
        ecs.world.insert_resource(Delta::default());

        // Add stages
        ecs.schedule
            .add_stage(
                Stages::Startup,
                Schedule::default()
                    .with_run_criteria(RunOnce::default())
                    .with_stage(Stages::Startup, SystemStage::parallel()),
            )
            .add_stage(Stages::Preupdate, SystemStage::parallel())
            .add_stage(Stages::Update, SystemStage::parallel())
            .add_stage(Stages::Postupdate, SystemStage::parallel());

        // Add systems
        ecs.schedule
            .stage(Stages::Startup, |schedule: &mut Schedule| {
                return schedule.add_system_to_stage(Stages::Startup, hello_world);
            })
            // .stage(Stages::Startup, |schedule: &mut Schedule| {
            //     return schedule.add_system_to_stage(Stages::Startup, spawn_system);
            // })
            .add_system_to_stage(Stages::Preupdate, paddle_system)
            //
            .add_system_to_stage(Stages::Update, collision_system)
            //
            .add_system_to_stage(Stages::Postupdate, render_system);

        ecs
    }

    #[export]
    fn _ready(&mut self, o: &Node2D) {
        let vis_server = unsafe { VisualServer::godot_singleton() };
        let project_settings = ProjectSettings::godot_singleton();

        let dummy_rid = Rid::new();

        let paddle_image = Image::new();
        paddle_image
            .load(project_settings.globalize_path("res://assets/Paddle.png"))
            .expect("Unable to load paddle image");
        let paddle_image = paddle_image.into_shared();
        let paddle_image = unsafe { paddle_image.assume_safe() };

        create_paddle(Paddle::Left, o, &mut self.world, vis_server, paddle_image);

        create_paddle(Paddle::Right, o, &mut self.world, vis_server, paddle_image);

        self.textures.push(paddle_image.claim());
    }

    #[export]
    fn _process(&mut self, _o: &Node2D, delta: f32) {
        let mut input_queue = self.world.get_resource_mut::<InputQueue>().unwrap();
        let input_handler = Input::godot_singleton();

        if input_handler.is_key_pressed(GlobalConstants::KEY_W) {
            input_queue.add(GodotInput::LeftUp);
        }
        if input_handler.is_key_pressed(GlobalConstants::KEY_S) {
            input_queue.add(GodotInput::LeftDown);
        }
        if input_handler.is_key_pressed(GlobalConstants::KEY_UP) {
            input_queue.add(GodotInput::RightUp);
        }
        if input_handler.is_key_pressed(GlobalConstants::KEY_DOWN) {
            input_queue.add(GodotInput::RightDown);
        }

        if input_handler.is_key_pressed(GlobalConstants::KEY_SPACE) {
            godot_print!("{:?}", self.textures);
        }

        let mut delta_res = self.world.get_resource_mut::<Delta>().unwrap();
        delta_res.0 = delta;
        self.schedule.run(&mut self.world);
    }
}

//region Systems

fn hello_world() {
    godot_print!("hello world");
}

fn paddle_system(
    mut input_queue: ResMut<InputQueue>,
    delta: Res<Delta>,
    mut query: Query<(&Paddle, &mut Drawable)>,
) {
    let vis_server = unsafe { VisualServer::godot_singleton() };

    let mut left_movement: f32 = 0.0;
    let mut right_movement: f32 = 0.0;

    for input in input_queue.read_all() {
        match input {
            GodotInput::LeftDown => left_movement += PADDLE_SPEED,
            GodotInput::LeftUp => left_movement -= PADDLE_SPEED,
            GodotInput::RightDown => right_movement += PADDLE_SPEED,
            GodotInput::RightUp => right_movement -= PADDLE_SPEED,
        }
    }

    for (p, mut d) in query.iter_mut() {
        match p {
            Paddle::Left => {
                if left_movement.abs() == 0.0 {
                    continue;
                }
                d.transform.m32 += left_movement * delta.0;
                vis_server.canvas_item_set_transform(d.rid, d.transform);
            }
            Paddle::Right => {
                if right_movement.abs() == 0.0 {
                    continue;
                }
                d.transform.m32 += right_movement * delta.0;
                vis_server.canvas_item_set_transform(d.rid, d.transform);
            }
        }
    }
}

fn collision_system() {}

fn render_system() {}

//endregion

fn create_paddle(
    paddle: Paddle,
    o: &Node2D,
    world: &mut World,
    vis_server: &VisualServer,
    paddle_image: TRef<Image>,
) {
    let paddle_rid = vis_server.canvas_item_create();
    let paddle_texture_rid = vis_server.texture_create_from_image(paddle_image, 7);

    let mut transform: Transform2D;
    match paddle {
        Paddle::Left => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, -500.0, 0.0);
        }
        Paddle::Right => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, 500.0, 0.0);
        }
    }

    let paddle_w = paddle_image.get_width();
    let paddle_h = paddle_image.get_height();

    transform.m31 -= paddle_w as f32;
    transform.m32 -= paddle_h as f32;

    vis_server.canvas_item_add_texture_rect(
        paddle_rid,
        Rect2::new(
            Point2::new(
                (paddle_image.get_width() / 2) as f32,
                (paddle_image.get_height() / 2) as f32,
            ),
            Size2::new(
                paddle_image.get_width() as f32,
                paddle_image.get_height() as f32,
            ),
        ),
        paddle_texture_rid,
        false,
        Color::rgba(1.0, 1.0, 1.0, 1.0),
        false,
        Rid::new(),
    );
    vis_server.canvas_item_set_parent(paddle_rid, o.get_canvas_item());
    vis_server.canvas_item_set_transform(paddle_rid, transform);
    world
        .spawn()
        .insert(Drawable {
            rid: paddle_rid,
            transform: transform,
        })
        .insert(paddle);
}
