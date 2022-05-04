use crate::prelude::*;
use mongodb::bson::doc;
use regex::Regex;
use rocket::serde::{Deserialize, Serialize};

// GitHub webhook support
#[derive(Deserialize)]
pub(crate) struct GHPingPayload {
    pub repository: GHRepo,
}

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
    pub binary: String,
}

// Final mod entry
#[derive(Serialize, Deserialize)]
pub(crate) struct ModEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub secret: String,
    pub search: String,
    pub published: i64,
    pub info: ModInfo,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<u32>,
    pub updated: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ModInfo {
    pub binary: String,
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
        fn trim_in_place(s: &mut String) {
            s.truncate(s.trim_end().len());
            s.drain(..(s.len() - s.trim_start().len()));
        }

        let time = timestamp();

        // Remove 'v' and 'V' prefix
        let v = &mut submission.version;
        v.drain(..(v.len() - v.trim_start_matches(['v', 'V']).len()));

        // Trim fields that should be trimmed
        trim_in_place(&mut submission.name);
        trim_in_place(&mut submission.owner);
        trim_in_place(&mut submission.description);
        trim_in_place(&mut submission.homepage);
        trim_in_place(&mut submission.version);
        trim_in_place(&mut submission.icon);
        trim_in_place(&mut submission.binary);

        if let Some(value) = errors(&submission) {
            return Err(value);
        }

        match process_binary(&mut submission.binary) {
            Ok(s) => submission.binary = s,
            Err(e) => return Err(e),
        }

        let id = format!("{}/{}", submission.owner, submission.name);

        Ok(Self {
            secret: submission.secret,
            search: n_gram(&id, 2),
            published: time,
            info: ModInfo {
                binary: submission.binary,
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

    if submission.name.is_empty() || submission.name.len() > 39 {
        Some("Name must be 1-39 bytes.")
    } else if submission.owner.is_empty() || submission.owner.len() > 39 {
        Some("Owner must be 1-39 bytes.")
    } else if submission.secret.is_empty() || submission.secret.len() > 500 {
        Some("Secret must be 1-500 bytes.")
    } else if submission.version.is_empty() || submission.version.len() > 50 {
        Some("Version must be 1-50 bytes.")
    } else if submission.description.len() > 500 {
        Some("Description must be 500 bytes or less.")
    } else if submission.homepage.len() > 500 {
        Some("Homepage URL must be 500 bytes or less.")
    } else if submission.icon.len() > 500 {
        Some("Icon URL must be 500 bytes or less.")
    } else if submission.binary.len() > 500 {
        Some("Binary URL must be 500 bytes or less.")
    } else if submission.name.contains(is_invalid) {
        Some("Name must match [a-zA-Z0-9_-.].")
    } else if submission.owner.contains(is_invalid) {
        Some("Owner must match [a-zA-Z0-9_-.].")
    } else if semver::Version::parse(&submission.version).is_err() {
        Some("Version must comply with https://semver.org.")
    } else if !submission.homepage.is_empty()
        && url::Url::parse(&submission.homepage)
            .and_then(|o| Ok(o.scheme() != "https"))
            .unwrap_or(true)
    {
        Some("Homepage must be a URL using the HTTPS scheme.")
    } else if url::Url::parse(&submission.icon)
        .and_then(|o| Ok(o.scheme() != "https"))
        .unwrap_or(true)
    {
        Some("Icon must be a URL using the HTTPS scheme.")
    } else {
        None
    }
}

fn timestamp() -> i64 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("???")
        .as_secs() as i64
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
