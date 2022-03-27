use crate::prelude::*;
use mongodb::bson::doc;
use regex::Regex;
use rocket::serde::{Deserialize, Serialize};

// GitHub webhook support
#[derive(Deserialize)]
pub(crate) struct GHRelPayload {
    pub action: String,
    pub repository: GHRepo,
    pub release: GHRelease,
}

#[derive(Deserialize)]
pub(crate) struct GHRepo {
    pub full_name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct GHRelease {
    pub tag_name: String,
    pub assets: Vec<GHAsset>,
}

#[derive(Deserialize)]
pub(crate) struct GHAsset {
    pub browser_download_url: String,
}

// Manual submissions
#[derive(Deserialize)]
pub(crate) struct Submission {
    pub name: String,
    pub owner: String,
    pub secret: String,
    pub description: String,
    pub homepage: String,
    pub version: String,
    pub icon: String,
    pub binaries: Vec<String>,
}

// Final mod entry
#[derive(Serialize, Deserialize)]
pub(crate) struct ModEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub secret: String,
    pub search: String,
    pub published: u32,
    pub info: ModInfo,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<u32>,
    pub updated: u32,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ModInfo {
    pub binaries: Vec<String>,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub icon: String,
}

pub enum Verification {
    Success,
    Failure,
    NotFound,
    Old,
    Error(mongodb::error::Error),
}

impl ModEntry {
    pub fn from_submission(mut submission: Submission) -> Result<Self, &'static str> {
        let time = timestamp();
        let id = format!("{}/{}", submission.owner.trim(), submission.name.trim());

        if let Some(value) = errors(&submission) {
            return Err(value);
        }

        for binary in &mut submission.binaries {
            match process_binary(&binary) {
                Ok(s) => *binary = s,
                Err(e) => return Err(e),
            }
        }

        Ok(Self {
            secret: submission.secret,
            search: n_gram(&id, 2),
            published: time,
            info: ModInfo {
                binaries: submission.binaries,
                version: submission.version,
                description: submission.description,
                homepage: submission.homepage,
                icon: submission.icon,
            },
            downloads: None,
            updated: time,
            id,
        })
    }

    pub async fn verify(&self, mods: &Mods) -> Verification {
        let query = doc! { "_id": &self.id };

        match mods.find_one(query, None).await {
            Ok(Some(e)) => {
                if e.secret != self.secret {
                    Failure
                } else if !self.newer(&e) {
                    Old
                } else {
                    Success
                }
            }
            Ok(None) => NotFound,
            Err(e) => {
                println!("Error while verifying mod {}: {e}", &self.id);
                Error(e)
            }
        }
    }

    fn newer(&self, other: &Self) -> bool {
        use semver::Version;

        match Version::parse(&self.info.version) {
            Ok(v1) => match Version::parse(&other.info.version) {
                Ok(v2) => v1 > v2,
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

fn process_binary(url: &str) -> Result<String, &'static str> {
    lazy_static::lazy_static! {
        // Capture 1 = google drive ID
        static ref DRIVE: Regex = Regex::new(r#"https://drive.google.com/file/d/([^/]+)"#).unwrap();
        static ref DRIVE_DIRECT: Regex = Regex::new(r#"https://drive.google.com/uc\?export=download&id=[^/]+"#).unwrap();
        static ref GITHUB: Regex = Regex::new(r#"https://github.com/Dual-Iron/.+/.+/download/.+?/[^/]+"#).unwrap();
        static ref DISCORD: Regex = Regex::new(r#"https://cdn.discordapp.com/attachments/\d+/\d+/[^/]+"#).unwrap();
    }
    if let Some(drive) = DRIVE.captures(url) {
        let id = &drive[1];
        Ok(format!(
            "https://drive.google.com/uc?export=download&id={id}"
        ))
    } else if let Some(other) = DRIVE_DIRECT.find(url) {
        Ok(url[other.range()].to_string())
    } else if let Some(other) = GITHUB.find(url) {
        Ok(url[other.range()].to_string())
    } else if let Some(other) = DISCORD.find(url) {
        Ok(url[other.range()].to_string())
    } else {
        Err("Each binary URL must be a Google Drive file, GitHub release asset, or Discord attachment.")
    }
}

fn errors(submission: &Submission) -> Option<&'static str> {
    fn is_invalid(c: char) -> bool {
        !c.is_alphanumeric() && !['.', '-', '_'].contains(&c)
    }

    if submission.binaries.is_empty() {
        Some("The submission has no binaries.")
    } else if submission.binaries.iter().any(|s| s.len() > 500) {
        Some("Binary length is too long (500 byte max).")
    } else if submission.name.len() > 39 {
        Some("Name is too long (39 byte max).")
    } else if submission.owner.len() > 39 {
        Some("Owner is too long (39 byte max).")
    } else if submission.secret.len() > 500 {
        Some("Secret length is too long (500 byte max).")
    } else if submission.version.len() > 50 {
        Some("Version length is too long (50 byte max).")
    } else if submission.description.len() > 500 {
        Some("Description length is too long (500 byte max).")
    } else if submission.homepage.len() > 500 {
        Some("Homepage length is too long (500 byte max).")
    } else if submission.name.contains(is_invalid) {
        Some("Name is invalid. Names must match [a-zA-Z0-9_-.].")
    } else if submission.owner.contains(is_invalid) {
        Some("Owner is invalid. Owners must match [a-zA-Z0-9_-.].")
    } else if semver::Version::parse(&submission.version).is_err() {
        Some("Version doesn't comply with https://semver.org.")
    } else {
        match url::Url::parse(&submission.icon) {
            Err(_) => Some("Icon is not a valid URL."),
            Ok(o) => (o.scheme() != "https").then(|| "Icon URL does not use HTTPS scheme."),
        }
    }
}

fn timestamp() -> u32 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("???")
        .as_secs() as u32
}

fn n_gram(s: &str, skip_n: usize) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    let mut ret = String::with_capacity(s.len());
    for word in s.split(['_', '-', '/']) {
        if word.graphemes(true).count() <= skip_n {
            ret.push_str(word);
            ret.push(' ');
        } else {
            for (byte, grapheme) in word.grapheme_indices(true).skip(skip_n) {
                ret.push_str(&word[..byte + grapheme.len()]);
                ret.push(' ');
            }
        }
    }
    ret
}
