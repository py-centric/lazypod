use lazypod::app::{App, CreateContainerForm, CreatePodForm, SearchImageForm, Tab};
use lazypod::podman::{Container, Image, Network, Pod, SearchResult, Volume};
use lazypod::ui;
use ratatui::{backend::TestBackend, Terminal};

fn setup_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create TestBackend terminal")
}

#[test]
fn test_ui_draw_empty_state() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw empty UI");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    // Verify Title bar and branding
    assert!(content.contains("Lazypod"));
    assert!(content.contains("Built by PyCentric"));
    assert!(content.contains("Running (0)"));
    assert!(content.contains("Stopped (0)"));
    assert!(content.contains("Images (0)"));
    assert!(content.contains("Volumes (0)"));
    assert!(content.contains("Networks (0)"));
    assert!(content.contains("Pods (0)"));
    assert!(content.contains("Nothing selected."));
    assert!(content.contains("Status: OK | PyCentric"));
}

#[test]
fn test_ui_draw_with_data() {
    let mut terminal = setup_test_terminal(140, 45);
    let mut app = App::new();

    app.running.push(Container {
        id: "c1234567890abcdef".to_string(),
        image: "redis:7-alpine".to_string(),
        command: Some(serde_json::Value::String(
            "docker-entrypoint.sh redis-server".to_string(),
        )),
        created: Some(serde_json::Value::Number(1_700_000_000.into())),
        state: Some(serde_json::Value::String("running".to_string())),
        status: Some(serde_json::Value::String("Up 2 hours".to_string())),
        names: Some(serde_json::Value::Array(vec!["my-redis".into()])),
        name: None,
        ports: Some(serde_json::Value::Array(vec![serde_json::json!({
            "HostIp": "0.0.0.0",
            "HostPort": "6379",
            "ContainerPort": 6379,
            "Protocol": "tcp"
        })])),
        pod_id: None,
        engine: "docker".to_string(),
    });

    app.images.push(Image {
        id: "sha256:abcdef123456".to_string(),
        parent_id: None,
        repo_tags: Some(serde_json::Value::Array(vec!["redis:7-alpine".into()])),
        repository: None,
        tag: None,
        names: None,
        size: Some(serde_json::json!(35_000_000)),
        created: Some(serde_json::Value::Number(1_700_000_000.into())),
        dangling: None,
        engine: "docker".to_string(),
    });

    app.volumes.push(Volume {
        name: "redis-data".to_string(),
        driver: "local".to_string(),
        mountpoint: "/var/lib/docker/volumes/redis-data/_data".to_string(),
        engine: "docker".to_string(),
    });

    app.networks.push(Network {
        name: "frontend-net".to_string(),
        id: "net123456".to_string(),
        driver: "bridge".to_string(),
        engine: "docker".to_string(),
    });

    app.pods.push(Pod {
        id: "pod123456789".to_string(),
        name: "backend-pod".to_string(),
        status: "Running".to_string(),
        created: Some(serde_json::Value::Number(1_700_000_000.into())),
        labels: None,
        num_containers: Some(0),
        engine: "podman".to_string(),
        containers: vec![],
    });

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw populated UI");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("my-redis"));
    assert!(content.contains("redis:7-alpine"));
    assert!(content.contains("redis-data"));
    assert!(content.contains("frontend-net"));
    assert!(content.contains("backend-pod"));
}

#[test]
fn test_ui_draw_help_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.show_help_tooltip = true;

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw help popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Help | Lazypod (Built by PyCentric)"));
    assert!(content.contains("Global Bindings"));
    assert!(content.contains("Toggle this help"));
}

#[test]
fn test_ui_draw_inspect_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.inspect_popup =
        Some("{\"Id\": \"test-id-123\", \"State\": {\"Running\": true}}".to_string());

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw inspect popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Inspect (Esc/g to close, j/k to scroll)"));
    assert!(content.contains("test-id-123"));
}

#[test]
fn test_ui_draw_create_container_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.create_container_form = Some(CreateContainerForm {
        name: "test-container".to_string(),
        command: "/bin/sh".to_string(),
        ports: "8080:80".to_string(),
        env: "ENV=prod".to_string(),
        active_field: 0,
    });

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw create container popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Create Container"));
    assert!(content.contains("test-container"));
    assert!(content.contains("8080:80"));
}

#[test]
fn test_ui_draw_create_pod_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.create_pod_form = Some(CreatePodForm {
        name: "test-pod".to_string(),
        network: "bridge".to_string(),
        share_pid: true,
        share_net: false,
        active_field: 2,
    });

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw create pod popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Create Pod"));
    assert!(content.contains("Share PID: [x]"));
    assert!(content.contains("Share Net: [ ]"));
}

#[test]
fn test_ui_draw_search_image_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.search_image_form = Some(SearchImageForm {
        query: "rust".to_string(),
        results: vec![SearchResult {
            index: "1".to_string(),
            name: "library/rust".to_string(),
            description: "Official Rust docker image".to_string(),
            stars: 999,
            official: "[OK]".to_string(),
        }],
        selected: 0,
        is_searching: false,
    });

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw search image popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Search Images"));
    assert!(content.contains("Query: rust"));
    assert!(content.contains("library/rust"));
}

#[test]
fn test_ui_draw_confirmation_popup() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.show_confirmation = true;
    app.pending_action = Some((
        Tab::Running,
        "docker".to_string(),
        "c123".to_string(),
        "stop".to_string(),
    ));

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw confirmation popup");

    let buffer = terminal.backend().buffer();
    let content = format!("{buffer:?}");

    assert!(content.contains("Confirmation"));
    assert!(content.contains("Are you sure you want to perform this action?"));
}

#[test]
fn test_ui_draw_images_tab_with_dangling_filter_and_history() {
    let mut terminal = setup_test_terminal(140, 40);
    let mut app = App::new();
    app.active_tab = Tab::Images;
    app.images = vec![
        Image {
            id: "sha256:111111".to_string(),
            parent_id: None,
            repo_tags: Some(serde_json::Value::Array(vec!["my-app:v1".into()])),
            repository: Some("my-app".into()),
            tag: Some("v1".into()),
            names: None,
            size: Some(serde_json::json!(50_000_000)),
            created: Some(serde_json::Value::Number(1_700_000_000.into())),
            dangling: None,
            engine: "podman".to_string(),
        },
        Image {
            id: "sha256:222222".to_string(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: Some(serde_json::Value::Array(vec!["<none>:<none>".into()])),
            size: Some(serde_json::json!(10_000_000)),
            created: Some(serde_json::Value::Number(1_700_000_000.into())),
            dangling: Some(true),
            engine: "podman".to_string(),
        },
    ];
    app.image_history = vec!["CMD [\"sh\"]".to_string(), "COPY . /app".to_string()];

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw images tab");

    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Images (2)"));
    assert!(content.contains("my-app:v1"));
    assert!(content.contains("[dangling]"));
    assert!(content.contains("Image History / Layers"));
    assert!(content.contains("COPY . /app"));

    // Now test with dangling filter enabled
    app.filter_dangling_images = true;
    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw images tab with dangling filter");

    let content_filtered = format!("{:?}", terminal.backend().buffer());
    assert!(content_filtered.contains("Images [Dangling] (1/2)"));
}

#[test]
fn test_ui_draw_tag_image_popup() {
    use lazypod::app::TagImageForm;

    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.active_tab = Tab::Images;
    app.images = vec![Image {
        id: "sha256:111111".to_string(),
        parent_id: None,
        repo_tags: Some(serde_json::Value::Array(vec!["alpine:3.18".into()])),
        repository: None,
        tag: None,
        names: None,
        size: Some(serde_json::json!(5_000_000)),
        created: None,
        dangling: None,
        engine: "podman".to_string(),
    }];
    app.tag_image_form = Some(TagImageForm {
        target_tag: "my-registry.local/alpine:custom".to_string(),
    });

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw tag image popup");

    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Tag Image"));
    assert!(content.contains("alpine:3.18"));
    assert!(content.contains("my-registry.local/alpine:custom"));
}

#[test]
fn test_ui_draw_prune_confirmation_popups() {
    let mut terminal = setup_test_terminal(120, 40);
    let mut app = App::new();
    app.show_confirmation = true;
    app.pending_action = Some((
        Tab::Images,
        "all".to_string(),
        "dangling".to_string(),
        "prune_dangling".to_string(),
    ));

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw prune dangling confirmation");

    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("prune dangling images"));

    app.pending_action = Some((
        Tab::Images,
        "all".to_string(),
        "all_unused".to_string(),
        "prune_all".to_string(),
    ));

    terminal
        .draw(|f| ui::draw(f, &mut app))
        .expect("Failed to draw prune all confirmation");

    let content_all = format!("{:?}", terminal.backend().buffer());
    assert!(content_all.contains("prune ALL unused images"));
}
