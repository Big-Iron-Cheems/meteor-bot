mod help;
mod moderation;
mod silly;
mod utility;

use crate::{
    commands::{help::*, moderation::*, silly::*, utility::*}, Data,
    Error,
};
use poise::Command;

pub fn get_commands() -> Vec<Command<Data, Error>> {
    vec![
        // Help commands
        faq(),
        installation(),
        logs(),
        old_versions(),
        // Moderation commands
        ban(),
        close(),
        mute(),
        unmute(),
        // Silly commands
        capybara(),
        cat(),
        dog(),
        monkey(),
        panda(),
        // Utility commands
        link(),
        stats(),
    ]
}
