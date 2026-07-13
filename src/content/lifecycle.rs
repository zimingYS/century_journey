use crate::shared::states::AppState;
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

/// 判断本次进入游戏状态是否包含内容重载请求。
///
/// 每个调用系统拥有独立的消息游标，因此不同层可以消费同一个请求。
pub fn content_reload_requested(mut requests: MessageReader<ContentReloadRequested>) -> bool {
    requests.read().next().is_some()
}

/// 注册内容生命周期消息和跨层调度顺序。
pub struct ContentLifecyclePlugin;

impl Plugin for ContentLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ContentReloadRequested>().configure_sets(
            OnEnter(AppState::InGame),
            (
                ContentReloadSet::Request,
                ContentReloadSet::Load,
                ContentReloadSet::Consumers,
            )
                .chain(),
        );
    }
}
