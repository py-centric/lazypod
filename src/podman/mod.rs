pub mod models;

use anyhow::Result;
#[allow(unused_imports)]
pub use models::{Container, Image, Network, Pod, PodContainer, PortMapping, SearchResult, Volume};
use tokio::process::Command;

#[async_trait::async_trait]
pub trait EngineClient: Send + Sync {
    async fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>>;
    async fn get_images(&self, engines: &[String]) -> Result<Vec<Image>>;
    async fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>>;
    async fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>>;
    async fn get_pods(&self, engines: &[String]) -> Result<Vec<Pod>>;
    async fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()>;
    async fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn action_pod(&self, engine: &str, id: &str, action: &str) -> Result<()>;
    async fn run_container(
        &self,
        engine: &str,
        image: &str,
        name: &str,
        ports: &str,
        env: &str,
        command: &str,
    ) -> Result<()>;
    async fn create_pod(&self, engine: &str, name: &str, network: &str, share: &str) -> Result<()>;
    async fn search_images(&self, engines: &[String], term: &str) -> Result<Vec<SearchResult>>;
    async fn pull_image(&self, engine: &str, image: &str) -> Result<()>;
    async fn get_container_logs(&self, engine: &str, id: &str) -> Result<Vec<String>>;
    async fn get_pod_logs(&self, engine: &str, pod_id: &str) -> Result<Vec<String>>;
    async fn get_container_inspect(&self, engine: &str, id: &str) -> Result<String>;
    async fn configure_registries(&self, registries_csv: &str) -> Result<()>;
}

pub struct LocalEngines;

async fn run_cmd_with_timeout(
    mut cmd: Command,
    timeout_ms: u64,
) -> std::io::Result<std::process::Output> {
    match tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), cmd.output()).await {
        Ok(res) => res,
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "engine command timed out",
        )),
    }
}

#[async_trait::async_trait]
impl EngineClient for LocalEngines {
    async fn get_containers(&self, engines: &[String]) -> Result<Vec<Container>> {
        let mut handles = Vec::new();
        for engine in engines {
            let engine = engine.clone();
            handles.push(tokio::spawn(async move {
                let mut cmd = Command::new(&engine);
                cmd.args(["ps", "-a", "--format", "json"]);
                // Podman: include --pod flag so PodID field is populated
                if engine == "podman" {
                    cmd.arg("--pod");
                }
                let output = run_cmd_with_timeout(cmd, 5000).await;
                (engine, output)
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(res) = handle.await {
                results.push(res);
            }
        }

        let mut all_containers = Vec::new();
        let mut errors = Vec::new();

        for (engine, output) in results {
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let mut parsed: Vec<Container> = parse_json_output(&out.stdout);
                        for item in &mut parsed {
                            item.engine.clone_from(&engine);
                        }
                        all_containers.extend(parsed);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stderr.trim().is_empty() {
                            tracing::warn!("{engine} ps error: {}", stderr.trim());
                            errors.push(format!("{engine}: {}", stderr.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute {engine}: {e}");
                    errors.push(format!("{engine}: {e}"));
                }
            }
        }

        if all_containers.is_empty() && !errors.is_empty() && errors.len() == engines.len() {
            return Err(anyhow::anyhow!("{}", errors.join("; ")));
        }

        Ok(all_containers)
    }

    async fn get_images(&self, engines: &[String]) -> Result<Vec<Image>> {
        let mut handles = Vec::new();
        for engine in engines {
            let engine = engine.clone();
            handles.push(tokio::spawn(async move {
                let mut cmd = Command::new(&engine);
                cmd.args(["images", "--format", "json"]);
                let output = run_cmd_with_timeout(cmd, 5000).await;
                (engine, output)
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(res) = handle.await {
                results.push(res);
            }
        }

        let mut all_images = Vec::new();
        let mut errors = Vec::new();

        for (engine, output) in results {
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let mut parsed: Vec<Image> = parse_json_output(&out.stdout);
                        for item in &mut parsed {
                            item.engine.clone_from(&engine);
                        }
                        all_images.extend(parsed);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stderr.trim().is_empty() {
                            tracing::warn!("{engine} images error: {}", stderr.trim());
                            errors.push(format!("{engine}: {}", stderr.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute {engine}: {e}");
                    errors.push(format!("{engine}: {e}"));
                }
            }
        }

        if all_images.is_empty() && !errors.is_empty() && errors.len() == engines.len() {
            return Err(anyhow::anyhow!("{}", errors.join("; ")));
        }

        Ok(all_images)
    }

    async fn get_volumes(&self, engines: &[String]) -> Result<Vec<Volume>> {
        let mut handles = Vec::new();
        for engine in engines {
            let engine = engine.clone();
            handles.push(tokio::spawn(async move {
                let mut cmd = Command::new(&engine);
                cmd.args(["volume", "ls", "--format", "json"]);
                let output = run_cmd_with_timeout(cmd, 5000).await;
                (engine, output)
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(res) = handle.await {
                results.push(res);
            }
        }

        let mut all_volumes = Vec::new();
        let mut errors = Vec::new();

        for (engine, output) in results {
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let mut parsed: Vec<Volume> = parse_json_output(&out.stdout);
                        for item in &mut parsed {
                            item.engine.clone_from(&engine);
                        }
                        all_volumes.extend(parsed);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stderr.trim().is_empty() {
                            tracing::warn!("{engine} volume ls error: {}", stderr.trim());
                            errors.push(format!("{engine}: {}", stderr.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute {engine}: {e}");
                    errors.push(format!("{engine}: {e}"));
                }
            }
        }

        if all_volumes.is_empty() && !errors.is_empty() && errors.len() == engines.len() {
            return Err(anyhow::anyhow!("{}", errors.join("; ")));
        }

        Ok(all_volumes)
    }

    async fn get_networks(&self, engines: &[String]) -> Result<Vec<Network>> {
        let mut handles = Vec::new();
        for engine in engines {
            let engine = engine.clone();
            handles.push(tokio::spawn(async move {
                let mut cmd = Command::new(&engine);
                cmd.args(["network", "ls", "--format", "json"]);
                let output = run_cmd_with_timeout(cmd, 5000).await;
                (engine, output)
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(res) = handle.await {
                results.push(res);
            }
        }

        let mut all_networks = Vec::new();
        let mut errors = Vec::new();

        for (engine, output) in results {
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let mut parsed: Vec<Network> = parse_json_output(&out.stdout);
                        for item in &mut parsed {
                            item.engine.clone_from(&engine);
                        }
                        all_networks.extend(parsed);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stderr.trim().is_empty() {
                            tracing::warn!("{engine} network ls error: {}", stderr.trim());
                            errors.push(format!("{engine}: {}", stderr.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute {engine}: {e}");
                    errors.push(format!("{engine}: {e}"));
                }
            }
        }

        if all_networks.is_empty() && !errors.is_empty() && errors.len() == engines.len() {
            return Err(anyhow::anyhow!("{}", errors.join("; ")));
        }

        Ok(all_networks)
    }

    async fn get_pods(&self, engines: &[String]) -> Result<Vec<Pod>> {
        let mut handles = Vec::new();
        for engine in engines {
            let engine = engine.clone();
            handles.push(tokio::spawn(async move {
                if engine == "docker" {
                    return (
                        engine,
                        Ok(std::process::Output {
                            status: std::os::unix::process::ExitStatusExt::from_raw(0),
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                        }),
                    );
                }
                let mut cmd = Command::new(&engine);
                cmd.args(["pod", "ps", "--format", "json"]);
                let output = run_cmd_with_timeout(cmd, 5000).await;
                (engine, output)
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(res) = handle.await {
                results.push(res);
            }
        }

        let mut all_pods = Vec::new();
        let mut errors = Vec::new();

        for (engine, output) in results {
            if engine == "docker" {
                continue;
            }
            match output {
                Ok(out) => {
                    if out.status.success() {
                        let mut parsed: Vec<Pod> = parse_json_output(&out.stdout);
                        for item in &mut parsed {
                            item.engine.clone_from(&engine);
                        }
                        all_pods.extend(parsed);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stderr.trim().is_empty() {
                            tracing::warn!("{engine} pod ps error: {}", stderr.trim());
                            errors.push(format!("{engine}: {}", stderr.trim()));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute {engine}: {e}");
                    errors.push(format!("{engine}: {e}"));
                }
            }
        }

        if all_pods.is_empty() && !errors.is_empty() && engines.contains(&"podman".to_string()) {
            return Err(anyhow::anyhow!("{}", errors.join("; ")));
        }

        Ok(all_pods)
    }

    async fn action_container(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        let output = Command::new(engine).args([action, id]).output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to {} container {}: {}",
                action,
                id,
                stderr.trim()
            ));
        }
        Ok(())
    }

    async fn action_image(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        let output = Command::new(engine)
            .args(["image", action, id])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to {} image {}: {}",
                action,
                id,
                stderr.trim()
            ));
        }
        Ok(())
    }

    async fn action_volume(&self, engine: &str, name: &str, action: &str) -> Result<()> {
        let output = Command::new(engine)
            .args(["volume", action, name])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to {} volume {}: {}",
                action,
                name,
                stderr.trim()
            ));
        }
        Ok(())
    }

    async fn action_network(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        let output = Command::new(engine)
            .args(["network", action, id])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to {} network {}: {}",
                action,
                id,
                stderr.trim()
            ));
        }
        Ok(())
    }

    async fn action_pod(&self, engine: &str, id: &str, action: &str) -> Result<()> {
        let output = Command::new(engine)
            .args(["pod", action, id])
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to {} pod {}: {}",
                action,
                id,
                stderr.trim()
            ));
        }
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
            if let Some(words) = shlex::split(env) {
                for e in words {
                    cmd.arg("-e").arg(e);
                }
            } else {
                for e in env.split_whitespace() {
                    cmd.arg("-e").arg(e);
                }
            }
        }

        cmd.arg(image);

        let command = command.trim();
        if !command.is_empty() {
            if let Some(words) = shlex::split(command) {
                for arg in words {
                    cmd.arg(arg);
                }
            } else {
                for arg in command.split_whitespace() {
                    cmd.arg(arg);
                }
            }
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to run container: {output:?}"));
        }
        Ok(())
    }

    async fn create_pod(&self, engine: &str, name: &str, network: &str, share: &str) -> Result<()> {
        let mut cmd = Command::new(engine);
        cmd.arg("pod").arg("create");

        let name = name.trim();
        if !name.is_empty() {
            cmd.arg("--name").arg(name);
        }

        let network = network.trim();
        if !network.is_empty() {
            cmd.arg("--network").arg(network);
        }

        let share = share.trim();
        if !share.is_empty() {
            cmd.arg("--share").arg(share);
        }

        let output = cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create pod: {}", stderr.trim()));
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

    async fn get_pod_logs(&self, engine: &str, pod_id: &str) -> Result<Vec<String>> {
        let output = Command::new(engine)
            .args(["pod", "logs", "--tail", "50", pod_id])
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

    async fn get_container_inspect(&self, engine: &str, id: &str) -> Result<String> {
        let output = Command::new(engine).args(["inspect", id]).output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to inspect {}: {}",
                id,
                stderr.trim()
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Try to pretty-print the JSON
        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(val) => {
                Ok(serde_json::to_string_pretty(&val).unwrap_or_else(|_| stdout.to_string()))
            }
            Err(_) => Ok(stdout.to_string()),
        }
    }

    async fn configure_registries(&self, registries_csv: &str) -> Result<()> {
        let registries_csv = registries_csv.to_string();
        tokio::task::spawn_blocking(move || {
            let registry_list: Vec<&str> = registries_csv
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            let formatted = registry_list
                .iter()
                .map(|s| format!("\"{s}\""))
                .collect::<Vec<String>>()
                .join(", ");

            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("~"));
            let containers_dir = std::path::PathBuf::from(home).join(".config/containers");
            std::fs::create_dir_all(&containers_dir).ok();

            let conf_path = containers_dir.join("registries.conf");
            let new_line = format!("unqualified-search-registries = [{formatted}]");

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
        })
        .await??;
        Ok(())
    }
}

#[must_use]
pub fn parse_json_output<T: serde::de::DeserializeOwned>(stdout: &[u8]) -> Vec<T> {
    if stdout.is_empty() {
        return vec![];
    }

    // Try parsing as a top-level JSON array first (Podman format: `[ {...}, {...} ]`)
    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(stdout) {
        if let Some(arr) = val.as_array() {
            return arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
        }
    }

    // Next try stream / newline-delimited JSON (Docker format: `{"ID":"1"}\n{"ID":"2"}\n`)
    let stream_items: Vec<T> = serde_json::Deserializer::from_slice(stdout)
        .into_iter::<T>()
        .filter_map(Result::ok)
        .collect();
    if !stream_items.is_empty() {
        return stream_items;
    }

    // Fallback: line-by-line parsing
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
        assert!(containers[0].is_running());
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
            ports: None,
            pod_id: None,
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
            ports: None,
            pod_id: None,
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
            ports: None,
            pod_id: None,
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
            ports: None,
            pod_id: None,
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

    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_container_get_ports() {
        use crate::podman::models::PortMapping;

        // Test array format (Docker-style)
        let c = Container {
            id: "1".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            ports: Some(serde_json::json!([
                { "IP": "0.0.0.0", "PrivatePort": 80, "PublicPort": 8080, "Type": "tcp" }
            ])),
            pod_id: None,
            engine: "test".into(),
        };
        let ports = c.get_ports();
        assert_eq!(ports.len(), 1);
        assert_eq!(
            ports[0],
            PortMapping {
                host_ip: "0.0.0.0".into(),
                host_port: 8080,
                container_port: 80,
                protocol: "tcp".into(),
            }
        );

        // Test string format (Podman-style)
        let c2 = Container {
            id: "2".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            ports: Some(serde_json::Value::String("0.0.0.0:8080->80/tcp".into())),
            pod_id: None,
            engine: "test".into(),
        };
        let ports2 = c2.get_ports();
        assert_eq!(ports2.len(), 1);
        assert_eq!(
            ports2[0],
            PortMapping {
                host_ip: "0.0.0.0".into(),
                host_port: 8080,
                container_port: 80,
                protocol: "tcp".into(),
            }
        );

        // Test multiple ports with IPv6
        let c3 = Container {
            id: "3".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            ports: Some(serde_json::Value::String(
                "0.0.0.0:8080->80/tcp, :::9090->9090/tcp".into(),
            )),
            pod_id: None,
            engine: "test".into(),
        };
        let ports3 = c3.get_ports();
        assert_eq!(ports3.len(), 2);
        assert_eq!(ports3[1].host_port, 9090);
        assert_eq!(ports3[1].container_port, 9090);

        // Test exposed-only port (no host binding)
        let c4 = Container {
            id: "4".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            ports: Some(serde_json::Value::String("80/tcp".into())),
            pod_id: None,
            engine: "test".into(),
        };
        let ports4 = c4.get_ports();
        assert_eq!(ports4.len(), 1);
        assert_eq!(ports4[0].host_port, 0);
        assert_eq!(ports4[0].container_port, 80);

        // Test no ports
        let c5 = Container {
            id: "5".into(),
            image: "test".into(),
            command: None,
            created: None,
            state: None,
            status: None,
            names: None,
            name: None,
            ports: None,
            pod_id: None,
            engine: "test".into(),
        };
        assert!(c5.get_ports().is_empty());

        // Test get_port_strings convenience
        let strings = c.get_port_strings();
        assert_eq!(strings, vec!["0.0.0.0:8080->80/tcp"]);
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
            size: Some(serde_json::json!(100)),
            created: Some(serde_json::Value::Number(1_678_901_234.into())),
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

        // Docker stream format (newline-delimited JSON objects)
        let stream = b"{\"name\": \"net1\", \"id\": \"1\"}\n{\"name\": \"net2\", \"id\": \"2\"}\n";
        let stream_nets: Vec<Network> = parse_json_output(stream);
        assert_eq!(stream_nets.len(), 2);
        assert_eq!(stream_nets[0].name, "net1");
        assert_eq!(stream_nets[1].name, "net2");

        // Docker images with string sizes in stream format
        let img_stream =
            b"{\"id\": \"i1\", \"Size\": \"50MB\"}\n{\"id\": \"i2\", \"Size\": \"1.2GB\"}\n";
        let images: Vec<Image> = parse_json_output(img_stream);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].get_size_str(), "50MB");
        assert_eq!(images[1].get_size_str(), "1.2GB");
    }
}
