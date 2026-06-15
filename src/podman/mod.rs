pub mod models;

use anyhow::Result;
pub use models::{Container, Image, Network, SearchResult, Volume};
use tokio::process::Command;

#[async_trait::async_trait]
pub trait EngineClient: Send + Sync {
    async fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>>;
    async fn get_images(&self, engines: &[String]) -> Result<Vec<Image>>;
    async fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>>;
    async fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>>;
    async fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn run_container(
        &self,
        engine: &str,
        image: &str,
        name: &str,
        ports: &str,
        env: &str,
        command: &str,
    ) -> Result<()>;
    async fn search_images(&self, engines: &[String], term: &str) -> Result<Vec<SearchResult>>;
    async fn pull_image(&self, engine: &str, image: &str) -> Result<()>;
    async fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()>;
    async fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn get_container_logs(&self, engine: &str, id: &str) -> Result<Vec<String>>;
    async fn configure_registries(&self, registries_csv: &str) -> Result<()>;
}

pub struct LocalEngines;

#[async_trait::async_trait]
impl EngineClient for LocalEngines {
    async fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>> {
        let mut all_containers = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["ps", "-a", "--format", "json"])
                .output()
                .await;

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

    async fn get_images(&self, engines: &[String]) -> Result<Vec<Image>> {
        let mut all_images = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["images", "--format", "json"])
                .output()
                .await;

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

    async fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>> {
        let mut all_volumes = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["volume", "ls", "--format", "json"])
                .output()
                .await;

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

    async fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>> {
        let mut all_networks = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["network", "ls", "--format", "json"])
                .output()
                .await;

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

    async fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args([action, id]).output().await?;
        Ok(())
    }

    async fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["image", action, id]).output().await?;
        Ok(())
    }

    async fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["volume", action, name]).output().await?;
        Ok(())
    }

    async fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        Command::new(engine).args(["network", action, id]).output().await?;
        Ok(())
    }

    async fn run_container(
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
            for arg in command.split_whitespace() {
                cmd.arg(arg);
            }
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to run container: {:?}", output));
        }
        Ok(())
    }

    async fn search_images(&self, engines: &[String], term: &str) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();
        for engine in engines {
            let output = Command::new(engine)
                .args(["search", term, "--format", "json"])
                .output()
                .await;

            if let Ok(out) = output {
                if out.status.success() {
                    let parsed: Vec<SearchResult> = parse_json_output(&out.stdout);
                    all_results.extend(parsed);
                }
            }
        }
        Ok(all_results)
    }

    async fn pull_image(&self, engine: &str, image: &str) -> Result<()> {
        Command::new(engine).args(["pull", image]).output().await?;
        Ok(())
    }

    async fn get_container_logs(&self, engine: &str, id: &str) -> Result<Vec<String>> {
        let output = Command::new(engine)
            .args(["logs", "--tail", "50", id])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut logs = Vec::new();
        for line in stdout.lines() {
            logs.push(line.to_string());
        }
        for line in stderr.lines() {
            logs.push(line.to_string());
        }
        Ok(logs)
    }

    async fn configure_registries(&self, registries_csv: &str) -> Result<()> {
        let registries_csv = registries_csv.to_string();
        tokio::task::spawn_blocking(move || {
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
            std::fs::create_dir_all(&containers_dir).ok();

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
                if let Some(line) = new_lines.last() {
                    if !line.is_empty() {
                        new_lines.push(String::new());
                    }
                }
                new_lines.push(new_line);
            }

            new_lines.push(String::new());

            std::fs::write(conf_path, new_lines.join("\n"))
        }).await??;
        Ok(())
    }
}

pub fn parse_json_output<T: serde::de::DeserializeOwned>(stdout: &[u8]) -> Vec<T> {
    if stdout.is_empty() {
        return vec![];
    }

    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(stdout) {
        if let Some(arr) = val.as_array() {
            return arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
        } else if let Ok(item) = serde_json::from_value::<T>(val) {
            return vec![item];
        }
    }

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
        let containers: Vec<Container> = parse_json_output(raw_json.as_bytes());
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id, "123");
        assert_eq!(containers[0].is_running(), true);
        assert_eq!(containers[0].get_names(), vec!["my_container"]);
        assert_eq!(containers[0].get_command(), "sh");
    }

    #[test]
    fn test_container_get_names_edge_cases() {
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

        c.name = None;
        assert_eq!(c.get_names(), vec!["names_entry"]);

        c.names = Some(serde_json::Value::String("string_name".into()));
        assert_eq!(c.get_names(), vec!["string_name"]);

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
            status: None,
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
            state: Some(serde_json::Value::String("Running".into())),
            status: None,
            names: None,
            name: None,
            engine: "test".into(),
        };
        assert!(c.is_running());

        c.state = Some(serde_json::Value::String("Up".into()));
        assert!(c.is_running());

        c.state = Some(serde_json::Value::String("Exited".into()));
        c.status = Some(serde_json::Value::String("Up 5 seconds".into()));
        assert!(c.is_running());

        c.status = Some(serde_json::Value::String("Exited (0)".into()));
        assert!(!c.is_running());
    }

    #[test]
    fn test_image_get_names_full() {
        let img = Image {
            id: "img1".into(),
            parent_id: None,
            repo_tags: Some(serde_json::Value::Array(vec!["tag1".into(), "tag2".into()])),
            repository: None,
            tag: None,
            names: Some(serde_json::Value::String("name1".into())),
            size: Some(100),
            created: Some(serde_json::Value::Number(1678901234.into())),
            engine: "test".into(),
        };
        let names = img.get_names();
        assert!(names.contains(&"name1".to_string()));
        assert!(names.contains(&"tag1".to_string()));
        assert!(names.contains(&"tag2".to_string()));
    }

    #[test]
    fn test_parse_json_output_robustness() {
        let empty: Vec<Network> = parse_json_output(b"");
        assert!(empty.is_empty());

        let invalid: Vec<Network> = parse_json_output(b"not json");
        assert!(invalid.is_empty());

        let partial = b"[{\"name\": \"ok\"}, \"bad\"]";
        let nets: Vec<Network> = parse_json_output(partial);
        assert_eq!(nets.len(), 1);
        assert_eq!(nets[0].name, "ok");
    }
}
