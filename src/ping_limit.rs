
use poise::serenity_prelude::*;
use regex::Regex;

pub fn extract_mentions(content: &str) -> Vec<UserId> {
    // Define the regex pattern for user mentions
    let re = Regex::new(r"<@(\d+)>").unwrap();

    // Find all matches and capture the IDs
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|id| id.as_str().parse().unwrap()))
        .collect()
}
