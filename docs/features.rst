Features
========

Lazypod offers a comprehensive feature set designed to manage both Docker and Podman environments directly from your terminal.

Dual Engine Support
-------------------
Seamlessly switch between managing Docker containers, Podman containers, or a combined view of both. This unified experience significantly improves workflows for developers managing complex hybrid setups.

Running & Stopped Containers
----------------------------
- **Real-Time Viewing**: See all running and stopped containers across your environments.
- **Action Executions**: Readily stop running containers or start stopped ones with quick, intuitive keybindings.
- **Detailed Information**: Access metadata, statuses, engine sources, and commands associated with containers instantly on the right-side detail pane.

Image Management
----------------
- **Image Listing**: Browse through downloaded container images effortlessly.
- **Direct Pulling**: Pull images directly using standard image names/tags via a dedicated UI modal.
- **Registry Search**: Interactively search online registries, browse detailed results including descriptions and star ratings, and pull selected options.
- **Configure Registries**: Directly modify and configure local unqualified search registries (especially useful for Podman users).

Interactive Logs & Shell Execution
----------------------------------
- **Embedded Logs**: View a live, scrollable embedded tail of the logs for the actively selected running or stopped container.
- **Custom Exec Commands**: Spawn a temporary subprocess to execute any custom command inside a running container.
- **Interactive Shell**: Drop immediately into a ``/bin/sh`` or ``/bin/bash`` shell on a running container without breaking your flow.

Volumes and Networks
--------------------
Browse active volumes and network definitions. Removing dangling or unused networks and volumes directly from the interface helps maintain a clean host environment.