use std::collections::HashMap;
use poise::serenity_prelude::UserId;

pub mod balance;
pub mod give;
pub mod wager;

pub(self) fn get_user_wealth_mut(users: &mut HashMap<UserId, usize>, id: UserId) -> &mut usize {
    if users.get(&id).is_none() {
        users.insert(id, 100);
    }

    users.get_mut(&id).unwrap()
}
