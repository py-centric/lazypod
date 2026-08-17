---
description: "Serde deserialization patterns for external CLI tool JSON outputs"
globs: ["src/podman/**/*.rs"]
always_on: true
---

# Serde CLI JSON Deserialization Guardrails

1. **Duplicate Field Hazard**:
   - In `serde`, `#[serde(alias = "...")]` registers candidate field names for the *same* struct field.
   - If an external CLI (such as Podman or Docker) emits payloads containing *both* keys (e.g., `"Size": 100` and `"VirtualSize": 100` in the same object), Serde will fail with `Error("duplicate field <name>")`.
   - **Rule**: Never use `alias` for keys that may appear simultaneously. Define dedicated `Option<T>` fields (e.g., `pub size: Option<Value>` and `pub virtual_size: Option<Value>`) and use helper methods (e.g. `get_size_str()`) for fallback resolution.
2. **Resilient JSON Output Parsing**:
   - Always implement multi-tier parsing in `parse_json_output`:
     1. Array parsing (`serde_json::from_slice`).
     2. NDJSON streaming (`serde_json::Deserializer::from_slice`).
     3. Line-by-line fallback to ignore preamble text (e.g. Docker wrapper notices or system banners).
