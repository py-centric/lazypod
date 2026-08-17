use crate::podman::SearchResult;

/// Form state for container creation popup.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CreateContainerForm {
    pub name: String,
    pub command: String,
    pub ports: String,
    pub env: String,
    pub active_field: usize, // 0: Name, 1: Command, 2: Ports, 3: Env
}

impl CreateContainerForm {
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 4;
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.checked_sub(1).unwrap_or(3);
    }
}

/// Form state for image search popup.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SearchImageForm {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub is_searching: bool,
}

/// Form state for direct image pull popup.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DirectPullForm {
    pub image: String,
}

/// Form state for unqualified registries configuration popup.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ConfigureRegistriesForm {
    pub registries: String,
}

/// Form state for interactive container exec popup.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExecForm {
    pub command: String,
}

/// Form state for pod creation popup.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CreatePodForm {
    pub name: String,
    pub network: String,
    pub share_pid: bool,
    pub share_net: bool,
    pub active_field: usize, // 0: Name, 1: Network, 2: Share PID, 3: Share Net
}

impl CreatePodForm {
    pub fn next_field(&mut self) {
        self.active_field = (self.active_field + 1) % 4;
    }

    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.checked_sub(1).unwrap_or(3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container_form_field_navigation() {
        let mut form = CreateContainerForm::default();
        assert_eq!(form.active_field, 0);
        form.next_field();
        assert_eq!(form.active_field, 1);
        form.next_field();
        assert_eq!(form.active_field, 2);
        form.next_field();
        assert_eq!(form.active_field, 3);
        form.next_field();
        assert_eq!(form.active_field, 0);
        form.prev_field();
        assert_eq!(form.active_field, 3);
    }

    #[test]
    fn test_create_pod_form_field_navigation() {
        let mut form = CreatePodForm::default();
        assert_eq!(form.active_field, 0);
        form.next_field();
        assert_eq!(form.active_field, 1);
        form.prev_field();
        assert_eq!(form.active_field, 0);
        form.prev_field();
        assert_eq!(form.active_field, 3);
    }
}
