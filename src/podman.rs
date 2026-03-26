use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Driver", default)]
    pub driver: String,
    #[serde(alias = "Mountpoint", default)]
    pub mountpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    #[serde(alias = "name", alias = "Name", default)]
    pub name: String,
    #[serde(alias = "id", alias = "Id", default)]
    pub id: String,
    #[serde(alias = "driver", alias = "Driver", default)]
    pub driver: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub trait PodmanClient {
    fn get_containers(&self) -> Result<Vec<Container>>;
    fn get_images(&self) -> Result<Vec<Image>>;
    fn get_volumes(&self) -> Result<Vec<Volume>>;
    fn get_networks(&self) -> Result<Vec<Network>>;
    fn action_container(&self, id: &str, action: &str) -> Result<()>;
    fn run_container(&self, image: &str, name: &str, ports: &str, command: &str) -> Result<()>;
    fn search_images(&self, term: &str) -> Result<Vec<SearchResult>>;
    fn pull_image(&self, image: &str) -> Result<()>;
    fn get_container_logs(&self, id: &str) -> Result<String>;
}

pub struct LocalPodman;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    #[serde(alias = "Id", alias = "id", default)]
    pub id: String,
    #[serde(alias = "Image", alias = "image", default)]
    pub image: String,
    #[serde(alias = "Command", alias = "command")]
    pub command: Option<serde_json::Value>,
    #[serde(alias = "Created", alias = "created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(alias = "State", alias = "state")]
    pub state: Option<serde_json::Value>,
    #[serde(alias = "Status", alias = "status")]
    pub status: Option<serde_json::Value>,
    #[serde(alias = "Names", alias = "names")]
    pub names: Option<serde_json::Value>,
    #[serde(alias = "Name", alias = "name")]
    pub name: Option<String>,
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
                return arr.iter().filter_map(|x| x.as_str().map(String::from)).collect();
            } else if let Some(s) = v.as_str() {
                return vec![s.to_string()];
            }
        }
        vec![]
    }

    pub fn get_command(&self) -> String {
        if let Some(v) = &self.command {
            if let Some(arr) = v.as_array() {
                return arr.iter().filter_map(|x| x.as_str().map(String::from)).collect::<Vec<_>>().join(" ");
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
        if state == "running" || state == "up" {
            return true;
        }
        let status = self.get_status_str().to_lowercase();
        if status.starts_with("up") {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    #[serde(alias = "Id", alias = "id", default)]
    pub id: String,
    #[serde(alias = "ParentId", alias = "parentId", default)]
    pub parent_id: Option<String>,
    #[serde(alias = "RepoTags", alias = "repoTags")]
    pub repo_tags: Option<serde_json::Value>,
    #[serde(alias = "Names", alias = "names")]
    pub names: Option<serde_json::Value>,
    #[serde(alias = "Size", alias = "size", default)]
    pub size: Option<i64>,
}

impl Image {
    pub fn get_names(&self) -> Vec<String> {
        let mut names = Vec::new();
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
}

impl PodmanClient for LocalPodman {
    fn get_containers(&self) -> Result<Vec<Container>> {
        let output = Command::new("podman")
            .args(&["ps", "-a", "--format", "json"])
            .output()
            .context("failed to execute podman ps")?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }

        // DEBUG: Write raw JSON to file so the AI agent can read it
        if let Ok(json_str) = String::from_utf8(output.stdout.clone()) {
            let _ = std::fs::write("podman_debug.json", json_str);
        }

        let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
        let containers = if let Some(arr) = json_val.as_array() {
            arr.iter().filter_map(|v| {
                match serde_json::from_value::<Container>(v.clone()) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        let _ = std::fs::write("podman_parse_error.txt", format!("Failed to parse: {}\nRaw: {}", e, v));
                        None
                    }
                }
            }).collect()
        } else if let Ok(c) = serde_json::from_value::<Container>(json_val) {
            vec![c]
        } else {
            vec![]
        };

        Ok(containers)
    }

    fn get_images(&self) -> Result<Vec<Image>> {
        let output = Command::new("podman")
            .args(&["images", "--format", "json"])
            .output()
            .context("failed to execute podman images")?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
        let images = if let Some(arr) = json_val.as_array() {
            arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect()
        } else if let Ok(i) = serde_json::from_value::<Image>(json_val) {
            vec![i]
        } else {
            vec![]
        };

        Ok(images)
    }

    fn get_volumes(&self) -> Result<Vec<Volume>> {
        let output = Command::new("podman")
            .args(&["volume", "ls", "--format", "json"])
            .output()?;
        if !output.status.success() {
            return Ok(vec![]);
        }
        let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
        let vols = if let Some(arr) = json_val.as_array() {
            arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect()
        } else if let Ok(v) = serde_json::from_value::<Volume>(json_val) {
            vec![v]
        } else {
            vec![]
        };
        Ok(vols)
    }

    fn get_networks(&self) -> Result<Vec<Network>> {
        let output = Command::new("podman")
            .args(&["network", "ls", "--format", "json"])
            .output()?;
        if !output.status.success() {
            return Ok(vec![]);
        }
        let json_val: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or(serde_json::Value::Null);
        let nets = if let Some(arr) = json_val.as_array() {
            arr.iter().filter_map(|v| serde_json::from_value(v.clone()).ok()).collect()
        } else if let Ok(n) = serde_json::from_value::<Network>(json_val) {
            vec![n]
        } else {
            vec![]
        };
        Ok(nets)
    }

    fn action_container(&self, id: &str, action: &str) -> Result<()> {
        Command::new("podman")
            .args(&[action, id])
            .output()?;
        Ok(())
    }

    fn run_container(&self, image: &str, name: &str, ports: &str, command: &str) -> Result<()> {
        let mut cmd = Command::new("podman");
        cmd.arg("run").arg("-d");
        
        let name = name.trim();
        if !name.is_empty() {
            cmd.arg("--name").arg(name);
        }
        
        let ports = ports.trim();
        if !ports.is_empty() {
            cmd.arg("-p").arg(ports);
        }
        
        cmd.arg(image);
        
        let command = command.trim();
        if !command.is_empty() {
            // Very simple split
            for arg in command.split_whitespace() {
                cmd.arg(arg);
            }
        }
        
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to run container: {:?}", output));
        }
        Ok(())
    }

    fn search_images(&self, term: &str) -> Result<Vec<SearchResult>> {
        let output = Command::new("podman")
            .args(&["search", term, "--format", "json"])
            .output()?;
        
        if !output.status.success() {
            return Ok(vec![]);
        }
        
        let results = serde_json::from_slice(&output.stdout).unwrap_or_default();
        Ok(results)
    }

    fn pull_image(&self, image: &str) -> Result<()> {
        Command::new("podman")
            .args(&["pull", image])
            .output()?;
        Ok(())
    }

    fn get_container_logs(&self, id: &str) -> Result<String> {
        let output = Command::new("podman")
            .args(&["logs", "--tail", "50", id])
            .output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        let mut logs = String::new();
        logs.push_str(&stdout);
        if !stdout.is_empty() && !stderr.is_empty() && !stdout.ends_with('\n') {
            logs.push('\n');
        }
        logs.push_str(&stderr);
        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_containers() {
        let raw_json = r#"[
            {
                "Id": "123",
                "Image": "alpine",
                "Command": ["sh"],
                "Created": 123456789,
                "State": "running",
                "Status": "Up 2 hours",
                "Names": ["my_container"]
            }
        ]"#;
        let containers: Vec<Container> = serde_json::from_str(raw_json).unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "123");
        assert_eq!(containers[0].is_running(), true);
        assert_eq!(containers[0].get_names(), vec!["my_container"]);
    }
}
