//! Username generator - creates memorable usernames in adjective-animal format
//! Examples: swift-falcon, brave-otter, calm-tiger

use rand::seq::SliceRandom;
use rand::Rng;

const ADJECTIVES: &[&str] = &[
    "swift", "brave", "calm", "dark", "eager", "fair", "gentle", "happy", "idle", "jolly", "keen",
    "lucky", "merry", "noble", "proud", "quick", "rapid", "sharp", "strong", "true", "vivid",
    "warm", "wise", "young", "zesty", "agile", "bold", "cool", "deft", "elite", "fast", "grand",
    "hale", "iron", "jade", "kind", "lush", "mild", "neat", "open", "pure", "quiet", "rare",
    "safe", "tall", "ultra", "vast", "wild", "amber", "azure", "coral", "cyber", "lunar", "neon",
    "pixel", "solar",
];

const ANIMALS: &[&str] = &[
    "falcon", "otter", "tiger", "wolf", "eagle", "hawk", "lion", "bear", "fox", "deer", "owl",
    "crow", "heron", "lynx", "puma", "raven", "shark", "whale", "dolphin", "panther", "jaguar",
    "cobra", "viper", "python", "crane", "finch", "robin", "swift", "wren", "duck", "goose",
    "swan", "seal", "walrus", "badger", "ferret", "mink", "stoat", "hare", "rabbit", "moose",
    "elk", "bison", "horse", "zebra", "giraffe", "hippo", "rhino", "koala", "panda", "lemur",
    "gecko", "iguana", "turtle", "frog", "newt",
];

/// Generate a random username in adjective-animal format
pub fn generate_username() -> String {
    let mut rng = rand::thread_rng();

    let adjective = ADJECTIVES.choose(&mut rng).unwrap_or(&"swift");
    let animal = ANIMALS.choose(&mut rng).unwrap_or(&"falcon");

    format!("{adjective}-{animal}")
}

/// Generate a username with a numeric suffix for collision avoidance
pub fn generate_username_with_suffix() -> String {
    let mut rng = rand::thread_rng();
    let base = generate_username();
    let suffix: u16 = rng.gen_range(10..100);
    format!("{base}-{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_username() {
        let username = generate_username();
        assert!(username.contains('-'));
        let parts: Vec<&str> = username.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(ANIMALS.contains(&parts[1]));
    }

    #[test]
    fn test_generate_username_with_suffix() {
        let username = generate_username_with_suffix();
        let parts: Vec<&str> = username.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[2].parse::<u16>().is_ok());
    }
}
