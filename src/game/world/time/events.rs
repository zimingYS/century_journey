//! 定义世界时钟跨越分钟、日期和节气边界时发出的领域事件。

use super::calendar::CalendarSnapshot;
use bevy::prelude::Message;

macro_rules! clock_message {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Message, Debug, Clone, Copy)]
        pub struct $name(
            /// 边界发生后的权威日历快照。
            pub CalendarSnapshot,
        );
    };
}

clock_message!(GameMinuteElapsed, "世界时钟跨越游戏分钟边界时发送的消息。");
clock_message!(GameHourElapsed, "世界时钟跨越游戏小时边界时发送的消息。");
clock_message!(GameDayElapsed, "世界时钟跨越游戏日边界时发送的消息。");
clock_message!(SolarTermChanged, "世界时钟跨越节气边界时发送的消息。");
clock_message!(SeasonChanged, "世界时钟跨越季节边界时发送的消息。");
clock_message!(GameYearElapsed, "世界时钟跨越游戏年边界时发送的消息。");
