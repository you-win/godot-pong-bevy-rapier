use std::collections::{vec_deque::Drain, HashMap, VecDeque};

use bevy_ecs::prelude::*;
use bevy_ecs::schedule::RunOnce;
use gdnative::api::{GlobalConstants, ImageTexture, ProjectSettings, StreamTexture, VisualServer};
use gdnative::prelude::*;
use rapier2d::prelude::{
    nalgebra, vector, ActiveCollisionTypes, ActiveEvents, CoefficientCombineRule, Collider,
    ColliderBuilder, ColliderHandle, ColliderSet, IntegrationParameters, InteractionGroups,
    Isometry, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};

use crate::physics::RapierWorld2D;

#[derive(Default)]
struct Delta(f32);

#[derive(Component)]
struct Drawable {
    rid: Rid,
    transform: Transform2D,
}

#[derive(Component)]
struct PhysicsBody {
    body: RigidBodyHandle,
    collider: ColliderHandle,
}

impl PhysicsBody {
    fn new(body: RigidBodyHandle, collider: ColliderHandle) -> Self {
        Self { body, collider }
    }
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
        // ecs.world.insert_resource(RapierWorld2D::new());

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

        // let physics_world = self
        //     .world
        //     .get_resource_mut::<RapierWorld2D>()
        //     .expect("Unable to get physics world");

        // let mut bodies = physics_world.bodies.borrow_mut();
        // let mut bodies = physics_world.bodies.write().expect("Unable to get bodies");
        let mut bodies = RigidBodySet::new();

        // let mut colliders = physics_world.colliders.borrow_mut();
        // let mut colliders = physics_world
        //     .colliders
        //     .write()
        //     .expect("Unable to get colliders");
        let mut colliders = ColliderSet::new();

        let (left_rid, left_texture_rid) = create_paddle(
            Paddle::Left,
            o,
            &mut self.world,
            vis_server,
            paddle_image,
            &mut bodies,
            &mut colliders,
        );

        self.rids.push(left_rid);
        self.rids.push(left_texture_rid);

        let (right_rid, right_texture_rid) = create_paddle(
            Paddle::Right,
            o,
            &mut self.world,
            vis_server,
            paddle_image,
            &mut bodies,
            &mut colliders,
        );

        self.rids.push(right_rid);
        self.rids.push(right_texture_rid);

        let ball_image = Image::new();
        ball_image
            .load(project_settings.globalize_path("res://assets/icon.png"))
            .expect("Unable to load ball image");
        let ball_image = ball_image.into_shared();
        let ball_image = unsafe { ball_image.assume_safe() };

        let (ball_rid, ball_texture_rid) = create_ball(
            o,
            &mut self.world,
            vis_server,
            ball_image,
            &mut bodies,
            &mut colliders,
        );

        self.rids.push(ball_rid);
        self.rids.push(ball_texture_rid);

        self.textures.push(paddle_image.claim());

        // let physics_world = RapierWorld2D::new(bodies, colliders);
        self.world
            .insert_resource(RapierWorld2D::new(bodies, colliders));
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
    mut physics_world: Res<RapierWorld2D>,
    delta: Res<Delta>,
    mut query: Query<(&Paddle, &mut Drawable, &mut PhysicsBody)>,
) {
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

    for (p, mut d, b) in query.iter_mut() {
        match p {
            Paddle::Left => {
                if left_movement.abs() == 0.0 {
                    continue;
                }
                d.transform.m32 += left_movement * delta.0;
            }
            Paddle::Right => {
                if right_movement.abs() == 0.0 {
                    continue;
                }
                d.transform.m32 += right_movement * delta.0;
            }
        }

        let mut bodies = physics_world.bodies.write().expect("Unable to get body");
        let body = bodies.get_mut(b.body).expect("No physics body found");
        body.set_position(
            Isometry::translation(d.transform.m31, d.transform.m32),
            true,
        );
    }
}

fn ball_system(
    delta: Res<Delta>,
    physics_world: Res<RapierWorld2D>,
    mut query: Query<(&Ball, &mut Drawable, &Velocity, &mut PhysicsBody)>,
) {
    // let vis_server = unsafe { VisualServer::godot_singleton() };

    // for (_, mut d, v, mut b) in query.iter_mut() {
    //     d.transform.m31 += v.0.x * delta.0;
    //     d.transform.m32 += v.0.y * delta.0;

    //     vis_server.canvas_item_set_transform(d.rid, d.transform);

    //     let mut bodies = physics_world.bodies.write().expect("Unable to get body");
    //     let body = bodies.get_mut(b.body).expect("No physics body found");

    //     body.set_position(
    //         Isometry::translation(d.transform.m31, d.transform.m32),
    //         true,
    //     );
    // }
}

fn collision_system(
    delta: Res<Delta>,
    physics_world: Res<RapierWorld2D>,
    mut query: Query<(&mut Drawable, &Ball, &PhysicsBody)>,
) {
    let mut pipeline = physics_world
        .pipeline
        .write()
        .expect("Unable to get pipeline");
    let mut islands = physics_world
        .islands
        .write()
        .expect("Unable to get island manager");
    let mut broad_phase = physics_world
        .broad_phase
        .write()
        .expect("Unable to get broad_phase");
    let mut narrow_phase = physics_world
        .narrow_phase
        .write()
        .expect("Unable to get narrow_phase");
    let mut bodies = physics_world.bodies.write().expect("Unable to get bodies");
    let mut colliders = physics_world
        .colliders
        .write()
        .expect("Unable to get colliders");
    let mut joints = physics_world.joints.write().expect("Unable to get joints");
    let mut ccd = physics_world.ccd.write().expect("Unable to get ccd");

    let gravity = vector![physics_world.gravity.x, physics_world.gravity.y];
    let mut integration_params = IntegrationParameters::default();
    integration_params.max_position_iterations = 1;
    integration_params.max_linear_correction = 1.0;
    integration_params.dt = delta.0;

    pipeline.step(
        &gravity,
        &integration_params,
        &mut islands,
        &mut broad_phase,
        &mut narrow_phase,
        &mut bodies,
        &mut colliders,
        &mut joints,
        &mut ccd,
        &(),
        &(),
    );

    for (mut d, _, b) in query.iter_mut() {
        let ball = bodies
            .get(b.body)
            .expect("Unable to get ball after collision");

        let pos = ball.position();

        d.transform.m31 = pos.translation.vector.x;
        d.transform.m32 = pos.translation.vector.y;
    }
}

fn render_system(query: Query<&mut Drawable>) {
    let vis_server = unsafe { VisualServer::godot_singleton() };

    for d in query.iter() {
        vis_server.canvas_item_set_transform(d.rid, d.transform);
    }
}

//endregion

fn create_kinematic_body(
    width: i64,
    height: i64,
    transform: &Transform2D,
    mut bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
) -> PhysicsBody {
    let body = RigidBodyBuilder::new_kinematic_position_based()
        .translation(vector![transform.m31, transform.m32])
        .build();
    let body_handle = bodies.insert(body);

    let collider = ColliderBuilder::cuboid((width / 2) as f32, (height / 2) as f32)
        .friction(0.0)
        .restitution(1.0)
        .build();
    let collider_handle = colliders.insert_with_parent(collider, body_handle, &mut bodies);

    PhysicsBody::new(body_handle, collider_handle)
}

fn create_dynamic_body(
    width: i64,
    height: i64,
    transform: &Transform2D,
    mut bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
) -> PhysicsBody {
    let body = RigidBodyBuilder::new_dynamic()
        .translation(vector![transform.m31, transform.m32])
        .build();
    let body_handle = bodies.insert(body);

    let collider = ColliderBuilder::cuboid((width / 2) as f32, (height / 2) as f32)
        .restitution(1.0)
        .friction(0.0)
        .restitution_combine_rule(CoefficientCombineRule::Multiply)
        .friction_combine_rule(CoefficientCombineRule::Multiply)
        .build();
    let collider_handle = colliders.insert_with_parent(collider, body_handle, &mut bodies);

    PhysicsBody::new(body_handle, collider_handle)
}

fn create_paddle(
    paddle: Paddle,
    o: &Node2D,
    world: &mut World,
    vis_server: &VisualServer,
    paddle_image: TRef<Image>,
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
) -> (Rid, Rid) {
    let paddle_rid = vis_server.canvas_item_create();
    let paddle_texture_rid = vis_server.texture_create_from_image(paddle_image, 7);

    let paddle_w = paddle_image.get_width();
    let paddle_h = paddle_image.get_height();

    let transform: Transform2D;

    match paddle {
        Paddle::Left => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, -500.0, 0.0);
        }
        Paddle::Right => {
            transform = Transform2D::new(1.0, 0.0, 0.0, 1.0, 500.0, 0.0);
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
        .insert(create_kinematic_body(
            paddle_w, paddle_h, &transform, bodies, colliders,
        ));

    (paddle_rid, paddle_texture_rid)
}

fn create_ball(
    o: &Node2D,
    world: &mut World,
    vis_server: &VisualServer,
    ball_image: TRef<Image>,
    bodies: &mut RigidBodySet,
    colliders: &mut ColliderSet,
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

    let physics_body = create_dynamic_body(ball_w, ball_h, &transform, bodies, colliders);
    bodies
        .get_mut(physics_body.body)
        .unwrap()
        .apply_impulse(vector![-1000000.0, 0.0], true);

    world
        .spawn()
        .insert(Drawable {
            rid: ball_rid,
            transform: transform,
        })
        .insert(Ball)
        .insert(Velocity(Vector2::new(-250.0, 0.0)))
        .insert(physics_body);

    (ball_rid, ball_texture_rid)
}
