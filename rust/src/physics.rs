use std::cell::RefCell;
use std::sync::RwLock;

use gdnative::prelude::*;
use rapier2d::prelude::*;

// pub struct RapierWorld2D {
//     gravity: Vector2,

//     pipeline: RefCell<PhysicsPipeline>,
//     broad_phase: RefCell<BroadPhase>,
//     narrow_phase: RefCell<NarrowPhase>,
//     pub bodies: RefCell<RigidBodySet>,
//     pub colliders: RefCell<ColliderSet>,
//     joints: RefCell<JointSet>,
//     ccd: RefCell<CCDSolver>,
// }

// impl RapierWorld2D {
//     pub fn new() -> Self {
//         Self {
//             gravity: Vector2::new(0.0, 0.0),

//             pipeline: RefCell::new(PhysicsPipeline::new()),
//             broad_phase: RefCell::new(BroadPhase::new()),
//             narrow_phase: RefCell::new(NarrowPhase::new()),
//             bodies: RefCell::new(RigidBodySet::new()),
//             colliders: RefCell::new(ColliderSet::new()),
//             joints: RefCell::new(JointSet::new()),
//             ccd: RefCell::new(CCDSolver::new()),
//         }
//     }
// }

pub struct RapierWorld2D {
    pub gravity: Vector2,

    pub pipeline: RwLock<PhysicsPipeline>,
    pub islands: RwLock<IslandManager>,
    pub broad_phase: RwLock<BroadPhase>,
    pub narrow_phase: RwLock<NarrowPhase>,
    pub bodies: RwLock<RigidBodySet>,
    pub colliders: RwLock<ColliderSet>,
    pub joints: RwLock<JointSet>,
    pub ccd: RwLock<CCDSolver>,
}

impl RapierWorld2D {
    // pub fn new() -> Self {
    //     Self {
    //         gravity: Vector2::new(0.0, 0.0),

    //         pipeline: RwLock::new(PhysicsPipeline::new()),
    //         broad_phase: RwLock::new(BroadPhase::new()),
    //         narrow_phase: RwLock::new(NarrowPhase::new()),
    //         bodies: RwLock::new(RigidBodySet::new()),
    //         colliders: RwLock::new(ColliderSet::new()),
    //         joints: RwLock::new(JointSet::new()),
    //         ccd: RwLock::new(CCDSolver::new()),
    //     }
    // }
    pub fn new(bodies: RigidBodySet, colliders: ColliderSet) -> Self {
        Self {
            gravity: Vector2::new(0.0, 0.0),

            pipeline: RwLock::new(PhysicsPipeline::new()),
            islands: RwLock::new(IslandManager::new()),
            broad_phase: RwLock::new(BroadPhase::new()),
            narrow_phase: RwLock::new(NarrowPhase::new()),
            bodies: RwLock::new(bodies),
            colliders: RwLock::new(colliders),
            joints: RwLock::new(JointSet::new()),
            ccd: RwLock::new(CCDSolver::new()),
        }
    }
}
