use std::collections::{vec_deque::Drain, HashMap, VecDeque};

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::RunOnce;
use gdnative::api::{GlobalConstants, ImageTexture, ProjectSettings, StreamTexture, VisualServer};
use gdnative::prelude::*;
use rapier2d::prelude::{
    nalgebra, vector, ActiveCollisionTypes, ActiveEvents, ColliderBuilder, InteractionGroups,
};

#[derive(Default)]
struct Delta(f32);

#[derive(Component)]
struct Drawable {
    rid: Rid,
    transform: Transform2D,
}

#[derive(Component)]
struct Collider {
    collider: rapier2d::prelude::Collider,
}

const PADDLE_SPEED: f32 = 500.0;

#[derive(Component)]
enum Paddle {
    Left,
    Right,
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vector2);

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
    rids: Vec<Rid>,
}

#[methods]
impl Ecs {
    fn new() -> Self {
        let mut ecs = Ecs {
            schedule: Schedule::default(),
            world: World::default(),
            textures: Vec::new(),
            rids: Vec::new(),
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
            .add_system_to_stage(Stages::Preupdate, paddle_system)
            .add_system_to_stage(Stages::Preupdate, ball_system)
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

        let paddle_image = Image::new();
        paddle_image
            .load(project_settings.globalize_path("res://assets/Paddle.png"))
            .expect("Unable to load paddle image");
        let paddle_image = paddle_image.into_shared();
        let paddle_image = unsafe { paddle_image.assume_safe() };

        let (left_rid, left_texture_rid) =
            create_paddle(Paddle::Left, o, &mut self.world, vis_server, paddle_image);

        self.rids.push(left_rid);
        self.rids.push(left_texture_rid);

        let (right_rid, right_texture_rid) =
            create_paddle(Paddle::Right, o, &mut self.world, vis_server, paddle_image);

        self.rids.push(right_rid);
        self.rids.push(right_texture_rid);

        let ball_image = Image::new();
        ball_image
            .load(project_settings.globalize_path("res://assets/icon.png"))
            .expect("Unable to load ball image");
        let ball_image = ball_image.into_shared();
        let ball_image = unsafe { ball_image.assume_safe() };

        let (ball_rid, ball_texture_rid) = create_ball(o, &mut self.world, vis_server, ball_image);

        self.rids.push(ball_rid);
        self.rids.push(ball_texture_rid);

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

        let mut delta_res = self.world.get_resource_mut::<Delta>().unwrap();
        delta_res.0 = delta;
        self.schedule.run(&mut self.world);
    }

    #[export]
    fn _exit_tree(&self, _: &Node2D) {
        let vis_server = unsafe { VisualServer::godot_singleton() };

        for rid in self.rids.iter() {
            vis_server.free_rid(*rid);
        }
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

fn ball_system(delta: Res<Delta>, mut query: Query<(&Ball, &mut Drawable, &Velocity)>) {
    let vis_server = unsafe { VisualServer::godot_singleton() };

    for (_, mut d, v) in query.iter_mut() {
        d.transform.m31 += v.0.x * delta.0;
        d.transform.m32 += v.0.y * delta.0;

        vis_server.canvas_item_set_transform(d.rid, d.transform);
    }
}

fn collision_system(
    delta: Res<Delta>,
    mut query: Query<(&Collider, &mut Drawable, Option<&mut Velocity>)>,
) {
}

fn render_system() {}

//endregion

fn create_paddle(
    paddle: Paddle,
    o: &Node2D,
    world: &mut World,
    vis_server: &VisualServer,
    paddle_image: TRef<Image>,
) -> (Rid, Rid) {
    let paddle_rid = vis_server.canvas_item_create();
    let paddle_texture_rid = vis_server.texture_create_from_image(paddle_image, 7);

    let paddle_w = paddle_image.get_width();
    let paddle_h = paddle_image.get_height();

    let transform: Transform2D;
    let mut collider = ColliderBuilder::cuboid((paddle_w / 2) as f32, (paddle_h / 2) as f32)
        .restitution(0.7)
        .collision_groups(InteractionGroups::new(0b0000, 0b0000))
        .solver_groups(InteractionGroups::new(0b0000, 0b0000))
        .active_collision_types(ActiveCollisionTypes::default())
        .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS);

    match paddle {
        Paddle::Left => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, -500.0, 0.0);
            collider = collider.translation(vector![-500.0 as f32, 0.0 as f32]);
        }
        Paddle::Right => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, 500.0, 0.0);
            collider = collider.translation(vector![500.0 as f32, 0.0 as f32]);
        }
    }

    vis_server.canvas_item_add_texture_rect(
        paddle_rid,
        Rect2::new(
            Point2::new(-(paddle_w / 2) as f32, -(paddle_h / 2) as f32),
            Size2::new(paddle_w as f32, paddle_h as f32),
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
        .insert(paddle)
        .insert(Collider {
            collider: collider.build(),
        });

    (paddle_rid, paddle_texture_rid)
}

fn create_ball(
    o: &Node2D,
    world: &mut World,
    vis_server: &VisualServer,
    ball_image: TRef<Image>,
) -> (Rid, Rid) {
    let ball_w = ball_image.get_width();
    let ball_h = ball_image.get_height();

    let transform = Transform2D::identity();

    let ball_rid = vis_server.canvas_item_create();
    let ball_texture_rid = vis_server.texture_create_from_image(ball_image, 7);

    vis_server.canvas_item_add_texture_rect(
        ball_rid,
        Rect2::new(
            Point2::new(-(ball_w / 2) as f32, -(ball_h / 2) as f32),
            Size2::new(ball_w as f32, ball_h as f32),
        ),
        ball_texture_rid,
        false,
        Color::rgba(1.0, 1.0, 1.0, 1.0),
        false,
        Rid::new(),
    );

    vis_server.canvas_item_set_parent(ball_rid, o.get_canvas_item());
    vis_server.canvas_item_set_transform(ball_rid, transform);

    let collider = ColliderBuilder::ball(64.0)
        .restitution(0.7)
        .collision_groups(InteractionGroups::new(0b0000, 0b0000))
        .solver_groups(InteractionGroups::new(0b0000, 0b0000))
        .active_collision_types(ActiveCollisionTypes::default())
        .active_events(ActiveEvents::CONTACT_EVENTS | ActiveEvents::INTERSECTION_EVENTS)
        .build();

    world
        .spawn()
        .insert(Drawable {
            rid: ball_rid,
            transform: transform,
        })
        .insert(Ball)
        .insert(Velocity(Vector2::new(-250.0, 0.0)))
        .insert(Collider { collider });

    (ball_rid, ball_texture_rid)
}
