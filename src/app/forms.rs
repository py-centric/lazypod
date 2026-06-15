use crate::podman::SearchResult;

#[derive(Default, Clone)]
pub struct CreateContainerForm {
    pub name: String,
    pub command: String,
    pub ports: String,
    pub env: String,
    pub active_field: usize, // 0: Name, 1: Command, 2: Ports, 3: Env
}

#[derive(Default, Clone)]
pub struct SearchImageForm {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub is_searching: bool,
}

#[derive(Default, Clone)]
pub struct DirectPullForm {
    pub image: String,
}

#[derive(Default, Clone)]
pub struct ConfigureRegistriesForm {
    pub registries: String,
}

#[derive(Default, Clone)]
pub struct ExecForm {
    pub command: String,
}
