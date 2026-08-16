//! 局部光照任务通道与结果载体。

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use bevy::prelude::*;

use crate::game::world::lighting::chunk_light::ChunkLight;
use crate::game::world::lighting::local_queue::LocalLightingQueue;
use crate::game::world::lighting::rebuild::{BlockLightSource, LightingWorldSnapshot};

/// 单次局部光照任务的原始结果载体。
pub(super) struct LocalLightingBuildResult {
    pub(super) session_id: u64,
    pub(super) content_revision: u64,
    pub(super) targets: Vec<IVec3>,
    pub(super) dependency_halo: i32,
    pub(super) snapshot: LightingWorldSnapshot,
    pub(super) lights: HashMap<IVec3, ChunkLight>,
    pub(super) sources: Vec<BlockLightSource>,
    pub(super) priority: bool,
    pub(super) sky_dirty: bool,
    pub(super) waited_ticks: u16,
    pub(super) elapsed: Duration,
}

/// 局部光照任务通道；限制为少量并发任务，连续编辑通过目标队列合并。
#[derive(Resource)]
pub(crate) struct LocalLightingBuildChannel {
    pub(super) sender: mpsc::Sender<LocalLightingBuildResult>,
    pub(super) receiver: Mutex<mpsc::Receiver<LocalLightingBuildResult>>,
    pub(super) in_flight: Arc<AtomicUsize>,
}

impl Default for LocalLightingBuildChannel {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Mutex::new(receiver),
            in_flight: Arc::new(AtomicUsize::new(0)),
        }
    }
}

/// 注册局部光照任务所需的会话期资源。
pub(crate) fn register_resources(app: &mut App) {
    app.init_resource::<LocalLightingQueue>()
        .init_resource::<LocalLightingBuildChannel>();
}
