use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    #[serde(alias = "Name", alias = "name", default)]
    pub name: String,
    #[serde(alias = "Driver", alias = "driver", default)]
    pub driver: String,
    #[serde(alias = "Mountpoint", alias = "mountpoint", default)]
    pub mountpoint: String,
    #[serde(skip)]
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    #[serde(alias = "name", alias = "Name", alias = "Name", default)]
    pub name: String,
    #[serde(alias = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(alias = "driver", alias = "Driver", default)]
    pub driver: String,
    #[serde(skip)]
    pub engine: String,
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

pub trait EngineClient {
    fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>>;
    fn get_images(&self, engines: &[String]) -> Result<Vec<Image>>;
    fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>>;
    fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>>;
    fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    fn run_container(
        &self,
        engine: &str,
        image: &str,
        name: &str,
        ports: &str,
        env: &str,
        command: &str,
    ) -> Result<()>;
    fn search_images(&self, engines: &[String], term: &str) -> Result<Vec<SearchResult>>;
    fn pull_image(&self, engine: &str, image: &str) -> Result<()>;
    fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()>;
    fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    fn get_container_logs(&self, engine: &str, id: &str) -> Result<String>;
    fn configure_registries(&self, registries_csv: &str) -> Result<()>;
}

pub struct LocalEngines;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    #[serde(alias = "Id", alias = "id", alias = "ID", default)]
    pub id: String,
    #[serde(alias = "Image", alias = "image", alias = "Image", default)]
    pub image: String,
    #[serde(alias = "Command", alias = "command", alias = "Command")]
    pub command: Option<serde_json::Value>,
    #[serde(alias = "Created", alias = "created", alias = "CreatedAt")]
    pub created: Option<serde_json::Value>,
    #[serde(alias = "State", alias = "state", alias = "State")]
    pub state: Option<serde_json::Value>,
    #[serde(alias = "Status", alias = "status", alias = "Status")]
    pub status: Option<serde_json::Value>,
    #[serde(alias = "Names", alias = "names", alias = "Names")]
    pub names: Option<serde_json::Value>,
    #[serde(alias = "Name", alias = "name")]
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
    #[serde(alias = "Id", alias = "id", alias = "ID", default)]
    pub id: String,
    #[serde(alias = "ParentId", alias = "parentId", alias = "ParentID", default)]
    pub parent_id: Option<String>,
    #[serde(alias = "RepoTags", alias = "repoTags", alias = "RepoTags")]
    pub repo_tags: Option<serde_json::Value>,
    #[serde(alias = "Names", alias = "names", alias = "Names")]
    pub names: Option<serde_json::Value>,
    #[serde(alias = "Size", alias = "size", alias = "Size", default)]
    pub size: Option<i64>,
    #[serde(skip)]
    pub engine: String,
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

impl EngineClient for LocalEngines {
    fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>> {
        let mut all_containers = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["ps", "-a", "--format", "json"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let mut parsed: Vec<Container> = parse_json_output(&out.stdout);
                    for item in &mut parsed {
                        item.engine = engine.clone();
                    }
                    all_containers.extend(parsed);
                }
            }
        }
        Ok(all_containers)
    }

    fn get_images(&self, engines: &[String]) -> Result<Vec<Image>> {
        let mut all_images = Vec::new();
        for engine in engines {
            // Docker often uses `docker images --format json`, but older versions might need `--format '{{json .}}'`
            // `--format json` works in newer Docker and Podman.
            let output = Command::new(engine)
                .args(["images", "--format", "json"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let mut parsed: Vec<Image> = parse_json_output(&out.stdout);
                    for item in &mut parsed {
                        item.engine = engine.clone();
                    }
                    all_images.extend(parsed);
                }
            }
        }
        Ok(all_images)
    }

    fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>> {
        let mut all_volumes = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["volume", "ls", "--format", "json"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let mut parsed: Vec<Volume> = parse_json_output(&out.stdout);
                    for item in &mut parsed {
                        item.engine = engine.clone();
                    }
                    all_volumes.extend(parsed);
                }
            }
        }
        Ok(all_volumes)
    }

    fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>> {
        let mut all_networks = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["network", "ls", "--format", "json"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let mut parsed: Vec<Network> = parse_json_output(&out.stdout);
                    for item in &mut parsed {
                        item.engine = engine.clone();
                    }
                    all_networks.extend(parsed);
                }
            }
        }
        Ok(all_networks)
    }

    fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args([action, id]).output()?;
        Ok(())
    }

    fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["image", action, id]).output()?;
        Ok(())
    }

    fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["volume", action, name]).output()?;
        Ok(())
    }

    fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["network", action, id]).output()?;
        Ok(())
    }

    fn run_container(
        &self,
        engine: &str,
        image: &str,
        name: &str,
        ports: &str,
        env: &str,
        command: &str,
    ) -> Result<()> {
        let mut cmd = Command::new(engine);
        cmd.arg("run").arg("-d");

        let name = name.trim();
        if !name.is_empty() {
            cmd.arg("--name").arg(name);
        }

        let ports = ports.trim();
        if !ports.is_empty() {
            cmd.arg("-p").arg(ports);
        }

        let env = env.trim();
        if !env.is_empty() {
            for e in env.split_whitespace() {
                cmd.arg("-e").arg(e);
            }
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

    fn search_images(&self, engines: &[String], term: &str) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["search", term, "--format", "json"])
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    let parsed: Vec<SearchResult> = parse_json_output(&out.stdout);
                    all_results.extend(parsed);
                }
            }
        }
        Ok(all_results)
    }

    fn pull_image(&self, engine: &str, image: &str) -> Result<()> {
        Command::new(engine).args(["pull", image]).output()?;
        Ok(())
    }

    fn get_container_logs(&self, engine: &str, id: &str) -> Result<String> {
        let output = Command::new(engine)
            .args(["logs", "--tail", "50", id])
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

    fn configure_registries(&self, registries_csv: &str) -> Result<()> {
        let registry_list: Vec<&str> = registries_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let formatted = registry_list
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<String>>()
            .join(", ");

        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
        let containers_dir = std::path::PathBuf::from(home).join(".config/containers");
        std::fs::create_dir_all(&containers_dir).ok(); // Ignore err if exists

        let conf_path = containers_dir.join("registries.conf");
        let new_line = format!("unqualified-search-registries = [{}]", formatted);

        let existing_content = std::fs::read_to_string(&conf_path).unwrap_or_default();
        let mut new_lines = Vec::new();
        let mut replaced = false;

        for line in existing_content.lines() {
            if line.trim().starts_with("unqualified-search-registries") {
                new_lines.push(new_line.clone());
                replaced = true;
            } else {
                new_lines.push(line.to_string());
            }
        }

        if !replaced {
            match new_lines.last() {
                Some(line) if !line.is_empty() => new_lines.push(String::new()),
                _ => {}
            }
            new_lines.push(new_line);
        }

        new_lines.push(String::new()); // trailing newline

        std::fs::write(conf_path, new_lines.join("\n"))?;
        Ok(())
    }
}

/// Helper function to parse either a JSON array (Podman) or JSON Lines (Docker) into a Vec of items
fn parse_json_output<T: serde::de::DeserializeOwned>(stdout: &[u8]) -> Vec<T> {
    if stdout.is_empty() {
        return vec![];
    }

    // Attempt to parse as array first
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(stdout) {
        if let Some(arr) = val.as_array() {
            return arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
        } else if let Ok(item) = serde_json::from_value::<T>(val) {
            return vec![item]; // Single obj fallback
        }
    }

    // Fallback: parse as JSON Lines (Docker)
    let text = String::from_utf8_lossy(stdout);
    let mut results = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.is_empty() {
            if let Ok(item) = serde_json::from_str::<T>(line) {
                results.push(item);
            }
        }
    }

    results
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
        assert_eq!(containers[0].get_command(), "sh");
    }

    #[test]
    fn test_container_get_names_edge_cases() {
        // Case 1: Both name and names present
        let mut c = Container {
            id: "1".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: Some(serde_json::Value::Array(vec!["names_entry".into()])),
            name: Some("name_entry".into()),
            engine: "test".into(),
        };
        assert_eq!(c.get_names(), vec!["name_entry"]);

        // Case 2: Only names present
        c.name = None;
        assert_eq!(c.get_names(), vec!["names_entry"]);

        // Case 3: names is a string (Docker style sometimes)
        c.names = Some(serde_json::Value::String("string_name".into()));
        assert_eq!(c.get_names(), vec!["string_name"]);

        // Case 4: Nothing
        c.names = None;
        assert!(c.get_names().is_empty());
    }

    #[test]
    fn test_container_get_command_variants() {
        let mut c = Container {
            id: "1".into(),
            image: "test".into(),
            command: Some(serde_json::Value::Array(vec!["ls".into(), "-l".into()])),
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            engine: "test".into(),
        };
        assert_eq!(c.get_command(), "ls -l");

        c.command = Some(serde_json::Value::String("ps aux".into()));
        assert_eq!(c.get_command(), "ps aux");

        c.command = None;
        assert_eq!(c.get_command(), "");
    }

    #[test]
    fn test_container_status_fallback() {
        let c = Container {
            id: "1".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None, // Missing status
            names: None,
            name: None,
            engine: "test".into(),
        };
        assert_eq!(c.get_status_str(), "");
        assert_eq!(c.get_state_str(), "unknown");
    }

    #[test]
    fn test_container_is_running_variants() {
        let mut c = Container {
            id: "1".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: Some("Running".into()),
            status: None,
            names: None,
            name: None,
            engine: "test".into(),
        };
        assert!(c.is_running());

        c.state = Some("Up".into());
        assert!(c.is_running());

        c.state = Some("Exited".into());
        c.status = Some("Up 5 seconds".into());
        assert!(c.is_running());

        c.status = Some("Exited (0)".into());
        assert!(!c.is_running());
    }

    #[test]
    fn test_image_get_names_full() {
        let img = Image {
            id: "img1".into(),
            parent_id: None,
            repo_tags: Some(serde_json::Value::Array(vec!["tag1".into(), "tag2".into()])),
            names: Some(serde_json::Value::String("name1".into())),
            size: Some(100),
            engine: "test".into(),
        };
        let names = img.get_names();
        assert!(names.contains(&"name1".to_string()));
        assert!(names.contains(&"tag1".to_string()));
        assert!(names.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_parse_json_output_robustness() {
        // Empty
        let empty: Vec<Network> = parse_json_output(b"");
        assert!(empty.is_empty());

        // Invalid JSON
        let invalid: Vec<Network> = parse_json_output(b"not json");
        assert!(invalid.is_empty());

        // Array with some invalid items
        let partial = b"[{\"name\": \"ok\"}, \"bad\"]";
        let nets: Vec<Network> = parse_json_output(partial);
        assert_eq!(nets.len(), 1);
        assert_eq!(nets[0].name, "ok");
    }
}
