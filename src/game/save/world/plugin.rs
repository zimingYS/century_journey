//! 组装世界元数据、区块队列和卸载保存流程。

use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::game::save::AutoSaveTimer;
use crate::game::save::world::chunk::load::{LoadQueue, process_load_queue_system};
use crate::game::save::world::chunk::queue::{SaveQueue, SaveWorker, process_save_queue_system};
use crate::game::save::world::runtime::auto_save::auto_save_on_unload_system;
use crate::game::save::world::runtime::world_load::{
    CachedBlockIdRemap, cache_level_data_on_enter,
};
use crate::shared::states::AppState;
use bevy::app::{App, Plugin, PostUpdate};
use bevy::prelude::{IntoScheduleConfigs, OnEnter, in_state};

/// 组装世界存档资源、加载队列和区块写入系统。
pub(in crate::game::save) struct WorldSavePlugin;

impl Plugin for WorldSavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveQueue::default())
            .init_resource::<SaveWorker>()
            .insert_resource(LoadQueue::default())
            .init_resource::<AutoSaveTimer>()
            .init_resource::<CachedBlockIdRemap>()
            .add_systems(
                OnEnter(AppState::InGame),
                cache_level_data_on_enter
                    .in_set(ContentReloadSet::Consumers)
                    .run_if(content_reload_requested),
            )
            .add_systems(
                PostUpdate,
                (process_save_queue_system, process_load_queue_system)
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                PostUpdate,
                auto_save_on_unload_system.run_if(in_state(AppState::InGame)),
            );
    }
}
