//! 组装客户端输入、渲染、表现反馈和界面插件。

use bevy::prelude::*;

use crate::client::effect::ClientEffectPlugin;
use crate::client::input::ClientInputPlugin;
use crate::client::interpolation::ClientInterpolationPlugin;
use crate::client::particle::ClientParticlePlugin;
use crate::client::player::ClientPlayerPlugin;
use crate::client::presentation::ClientPresentationPlugin;
use crate::client::renderer::ClientRenderingPlugin;
use crate::client::sky::SkyPlugin;
use crate::client::sound::ClientSoundPlugin;
use crate::client::ui::UIPlugin;

/// Client 层插件聚合入口。
///
/// 本插件只组装本地输入和表现系统，不能拥有或直接实现权威玩法规则。
pub struct ClientPluginGroup;

impl Plugin for ClientPluginGroup {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ClientInputPlugin,
            ClientRenderingPlugin,
            ClientPlayerPlugin,
            ClientInterpolationPlugin,
            ClientPresentationPlugin,
            SkyPlugin,
            UIPlugin,
            ClientSoundPlugin,
            ClientParticlePlugin,
            ClientEffectPlugin,
        ));
    }
}
