mod help;
mod moderation;
mod silly;
mod utility;

use crate::{Data, Error};
use poise::Command;

pub fn get_commands() -> Vec<Command<Data, Error>> {
    vec![
        // --- Slash commands ---

        // Help commands
        help::faq(),
        help::installation(),
        help::logs(),
        help::old_versions(),
        // Moderation commands
        moderation::ban(),
        moderation::close(),
        moderation::mute(),
        moderation::unmute(),
        // Silly commands
        silly::capybara(),
        silly::cat(),
        silly::dog(),
        silly::monkey(),
        silly::panda(),
        // Utility commands
        utility::link(),
        utility::stats(),
        // --- Context menu commands ---

        // Moderation commands
        moderation::menu_ban(),
        moderation::menu_mute(),
        moderation::menu_unmute(),
    ]
}
