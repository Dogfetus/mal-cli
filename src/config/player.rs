use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Player {
    /// Prevent the regular playback method and use an external player instead
    #[serde(default)]
    pub disable_default_player: bool,

    /// allways marks the episode as watched after pressing play
    #[serde(default)]
    pub always_complete_episode: bool,

    /// Hook to run before playback starts
    /// Replaces: {title}, {episode}
    pub pre_playback_hook: Option<String>,

    /// Hook to run after playback ends
    /// Replaces: {url}, {referrer}, {title}, {episode}
    /// referrer might be empty
    pub post_playback_hook: Option<String>,
}
