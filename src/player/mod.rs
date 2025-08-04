use ratatui::prelude::CrosstermBackend;
use ratatui::DefaultTerminal;
use ratatui::Terminal;
use regex::Captures;
use regex::Regex;

use crate::mal::models::anime::Anime;
use std::io::ErrorKind;
use std::io::Stdout;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Clone)]
pub enum PlayError {
    NotReleased(Anime),
    CommandFailed {
        stderr: String,
        exit_code: i32,
        stdout: String,
    },
    NotFound(String),
    NoResults(String),
    Other(String),
}

#[derive(Debug)]
pub struct PlayResult {
    pub episode: u32,
    pub current_time: String,
    pub total_time: String,
    pub percentage: u8,
    pub fully_watched: bool,
    pub completed: bool,
}

pub struct AnimePlayer {
    ansi_regex: Regex,

    //mpv regex:
    av_regex: Regex,
    exit_regex: Regex,
}

impl std::fmt::Display for PlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayError::NotReleased(anime) => write!(
                f,
                "\"{}\"\nis not yet released.",
                if anime.alternative_titles.en.is_empty() {
                    anime.title.clone()
                } else {
                    anime.alternative_titles.en.clone()
                }
            ),
            PlayError::CommandFailed {
                stderr,
                exit_code,
                stdout,
            } => {
                write!(
                    f,
                    "ani-cli replied:\nError: {}\nExit code: {}\nOutput: {}",
                    stderr, exit_code, stdout
                )
            }
            PlayError::NotFound(msg) => write!(f, "Can't seem to find ani-cli: \n{}", msg),
            PlayError::NoResults(msg) => write!(
                f,
                "ani-cli replied:\nError: {}\nthe anime might not be available yet",
                msg
            ),
            PlayError::Other(msg) => write!(f, "Error running ani-cli: \n{}", msg),
        }
    }
}

impl AnimePlayer {
    pub fn new() -> Self {
        AnimePlayer {
            ansi_regex: Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]|\x1b\([AB]|\r|\x1b[78]").unwrap(),
            av_regex: Regex::new(r"AV: (\d{2}:\d{2}:\d{2}) / (\d{2}:\d{2}:\d{2}) \((\d+)%\)")
                .unwrap(),
            exit_regex: Regex::new(r"Exiting\.\.\. \((.*?)\)").unwrap(),
        }
    }

    pub fn extract_play_info(&self, stdout: &str, episode: u32) -> Option<PlayResult> {
        let last_av = if let Some(last_av) = stdout.rfind("AV: ") {
            let last_stdout = &stdout[last_av..];
            self.av_regex.captures(last_stdout)?
        } else {
            return None;
        };

        let exit_reason = self
            .exit_regex
            .captures(stdout)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str());

        let percentage = last_av[3].parse().unwrap_or(0);

        Some(PlayResult {
            current_time: last_av[1].to_string(),
            total_time: last_av[2].to_string(),
            completed: percentage >= 90,
            fully_watched: exit_reason == Some("End of file"),
            percentage,
            episode,
        })
    }

    pub fn play_anime(&self, anime: &Anime) -> Result<PlayResult, PlayError> {
        if anime.status == "upcoming" {
            return Err(PlayError::NotReleased(anime.clone()));
        }

        ratatui::restore();

        let next_episode = std::cmp::min(
            anime.my_list_status.num_episodes_watched + 1,
            anime.num_episodes,
        );

        // call ani-cli to play the anime TODO: change this
        let output = Command::new("ani-cli")
            .arg("--no-detach")
            .arg("--exit-after-play")
            .arg("-e")
            .arg(&next_episode.to_string())
            .arg(&anime.title)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| {
                if e.kind() == ErrorKind::NotFound {
                    PlayError::NotFound("ani-cli is not installed or not found in PATH".to_string())
                } else {
                    PlayError::Other(format!("Error running ani-cli: \n{}", e))
                }
            })?;

        let messy_stdout = String::from_utf8_lossy(&output.stdout);
        let messy_stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = self.ansi_regex.replace_all(&messy_stdout, "").to_string();
        let stderr = self.ansi_regex.replace_all(&messy_stderr, "").to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        if !stderr.is_empty() && exit_code != 0{
            if stderr.contains("No results found!") {
                return Err(PlayError::NoResults(stderr));
            } else {
                return Err(PlayError::CommandFailed {
                    stderr,
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout,
                });
            }
        }

        self.extract_play_info(&stdout, next_episode).ok_or_else(|| {
            PlayError::Other("ani-cli did not return any play information".to_string())
        })
    }
}
