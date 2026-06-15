use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volume {
    #[serde(rename = "name", alias = "Name", default)]
    pub name: String,
    #[serde(rename = "driver", alias = "Driver", default)]
    pub driver: String,
    #[serde(rename = "mountpoint", alias = "Mountpoint", default)]
    pub mountpoint: String,
    #[serde(skip)]
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Network {
    #[serde(rename = "name", alias = "Name", default)]
    pub name: String,
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "driver", alias = "Driver", default)]
    pub driver: String,
    #[serde(skip)]
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    #[serde(alias = "Index", default)]
    pub index: String,
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Description", default)]
    pub description: String,
    #[serde(alias = "Stars", default)]
    pub stars: u32,
    #[serde(alias = "Official", default)]
    pub official: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container {
    #[serde(rename = "id", alias = "Id", alias = "ID", alias = "id", default)]
    pub id: String,
    #[serde(rename = "image", alias = "Image", alias = "image", default)]
    pub image: String,
    #[serde(rename = "command", alias = "Command", alias = "command", default)]
    pub command: Option<serde_json::Value>,
    #[serde(rename = "created", alias = "Created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(rename = "state", alias = "State", alias = "state", default)]
    pub state: Option<serde_json::Value>,
    #[serde(rename = "status", alias = "Status", alias = "status", default)]
    pub status: Option<serde_json::Value>,
    #[serde(rename = "names", alias = "Names", alias = "names", default)]
    pub names: Option<serde_json::Value>,
    #[serde(rename = "name", alias = "Name", alias = "name", default)]
    pub name: Option<String>,
    #[serde(skip)]
    pub engine: String,
}

impl Container {
    pub fn get_names(&self) -> Vec<String> {
        if let Some(n) = &self.name {
            if !n.is_empty() {
                return vec![n.clone()];
            }
        }
        if let Some(v) = &self.names {
            if let Some(arr) = v.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect();
            } else if let Some(s) = v.as_str() {
                return vec![s.to_string()];
            }
        }
        vec![]
    }

    pub fn get_command(&self) -> String {
        if let Some(v) = &self.command {
            if let Some(arr) = v.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join(" ");
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        String::new()
    }

    pub fn get_state_str(&self) -> String {
        if let Some(s) = &self.state {
            if let Some(st) = s.as_str() {
                return st.to_string();
            }
        }
        "unknown".into()
    }

    pub fn get_status_str(&self) -> String {
        if let Some(s) = &self.status {
            if let Some(st) = s.as_str() {
                return st.to_string();
            }
        }
        "".into()
    }

    pub fn is_running(&self) -> bool {
        let state = self.get_state_str().to_lowercase();
        let status = self.get_status_str().to_lowercase();

        if state == "running" || state == "up" || status.starts_with("up") {
            return true;
        }
        if state == "exited" || state == "stopped" || state == "created" || state == "paused" {
            return false;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "parentId", alias = "ParentId", alias = "ParentID", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "repoTags", alias = "RepoTags", default)]
    pub repo_tags: Option<serde_json::Value>,
    #[serde(rename = "repository", alias = "Repository", default)]
    pub repository: Option<String>,
    #[serde(rename = "tag", alias = "Tag", default)]
    pub tag: Option<String>,
    #[serde(rename = "names", alias = "Names", default)]
    pub names: Option<serde_json::Value>,
    #[serde(rename = "size", alias = "Size", default)]
    pub size: Option<i64>,
    #[serde(rename = "created", alias = "Created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(skip)]
    pub engine: String,
}

impl Image {
    pub fn get_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(r) = &self.repository {
            if let Some(t) = &self.tag {
                names.push(format!("{}:{}", r, t));
            } else {
                names.push(r.clone());
            }
        }
        if let Some(v) = &self.names {
            if let Some(arr) = v.as_array() {
                names.extend(arr.iter().filter_map(|x| x.as_str().map(String::from)));
            } else if let Some(s) = v.as_str() {
                names.push(s.to_string());
            }
        }
        if let Some(v) = &self.repo_tags {
            if let Some(arr) = v.as_array() {
                names.extend(arr.iter().filter_map(|x| x.as_str().map(String::from)));
            } else if let Some(s) = v.as_str() {
                names.push(s.to_string());
            }
        }
        names
    }

    pub fn get_size_str(&self) -> String {
        let size = self.size.unwrap_or(0);
        if size == 0 {
            return "0 B".to_string();
        }
        let units = ["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_idx = 0;
        while size >= 1024.0 && unit_idx < units.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        format!("{:.2} {}", size, units[unit_idx])
    }

    pub fn get_created_str(&self) -> String {
        if let Some(v) = &self.created {
            if let Some(n) = v.as_i64() {
                // Assume Unix timestamp
                let d = UNIX_EPOCH + Duration::from_secs(n as u64);
                let datetime: DateTime<Utc> = d.into();
                return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        "Unknown".to_string()
    }
}
