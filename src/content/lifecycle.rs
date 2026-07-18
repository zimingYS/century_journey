use crate::shared::states::AppState;
use crate::{content::validation::ContentCompilation, engine::asset::manager::AssetManager};
use bevy::prelude::*;

/// 请求重新构建数据驱动内容注册表。
///
/// App 只发出生命周期消息，Content、Game 和 Client 各自刷新本层资源，
/// 从而避免底层模块读取 App 的会话实现。
#[derive(Message, Debug, Clone, Copy, Default)]
pub struct ContentReloadRequested;

/// 内容重载在进入游戏状态时的固定执行阶段。
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentReloadSet {
    Request,
    Load,
    Consumers,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentStartupSet {
    Compile,
    Registry,
    Assets,
}

/// 判断本次进入游戏状态是否包含内容重载请求。
///
/// 每个调用系统拥有独立的消息游标，因此不同层可以消费同一个请求。
pub fn content_reload_requested(mut requests: MessageReader<ContentReloadRequested>) -> bool {
    requests.read().next().is_some()
}

pub fn content_compilation_valid(compilation: Option<Res<ContentCompilation>>) -> bool {
    compilation.is_some_and(|compilation| compilation.is_valid())
}

fn compile_content_system(world: &mut World) {
    let compilation = {
        let asset = world.resource::<AssetManager>();
        crate::content::validation::compile_content(asset.resolver())
    };
    if compilation.is_valid() {
        info!(
            "[Content] compiled {} files into deterministic registries",
            compilation.report.checked_files
        );
    } else {
        error!(
            "[Content] compilation failed with {} error(s)",
            compilation.report.errors.len()
        );
        for diagnostic in &compilation.report.errors {
            error!("[Content] {diagnostic}");
        }
    }
    world.insert_resource(compilation);
}

/// 注册内容生命周期消息和跨层调度顺序。
pub struct ContentLifecyclePlugin;

impl Plugin for ContentLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ContentReloadRequested>()
            .configure_sets(
                OnEnter(AppState::Loading),
                (
                    ContentStartupSet::Compile,
                    ContentStartupSet::Registry,
                    ContentStartupSet::Assets,
                )
                    .chain(),
            )
            .configure_sets(
                OnEnter(AppState::InGame),
                (
                    ContentReloadSet::Request,
                    ContentReloadSet::Load,
                    ContentReloadSet::Consumers,
                )
                    .chain(),
            )
            .add_systems(
                OnEnter(AppState::Loading),
                compile_content_system.in_set(ContentStartupSet::Compile),
            );
    }
}
