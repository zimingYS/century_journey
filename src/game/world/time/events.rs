use super::calendar::CalendarSnapshot;
use bevy::prelude::Message;

macro_rules! clock_message {
    ($name:ident) => {
        #[derive(Message, Debug, Clone, Copy)]
        pub struct $name(pub CalendarSnapshot);
    };
}

clock_message!(GameMinuteElapsed);
clock_message!(GameHourElapsed);
clock_message!(GameDayElapsed);
clock_message!(SolarTermChanged);
clock_message!(SeasonChanged);
clock_message!(GameYearElapsed);
