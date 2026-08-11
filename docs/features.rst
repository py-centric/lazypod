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
- **Port Display**: View exposed and mapped ports for each container directly in the details panel.

Pod Management
--------------
Lazypod includes a dedicated **Pods** tab for managing Podman pods. Pods are groups of one or more containers that share resources, and are a core concept in Kubernetes-style workflows.

- **View Pods**: Browse all pods with their names, status, and creation time in the Pods panel.
- **Pod Details**: Select a pod to see its full details, including the list of member containers, their individual statuses, and labels.
- **Create Pods**: Press ``P`` on the Pods tab to create a new pod with a custom name.
- **Delete Pods**: Press ``d`` or ``Delete`` to remove a pod (with confirmation).
- **Port Display**: View the port mappings exposed by containers within a pod.

.. note::

   Pod management is a Podman-specific feature. Pods will not appear when filtering to Docker-only mode.

Port Display
------------
Lazypod displays port information for both containers and pods wherever available:

- **Containers**: The details panel shows port mappings (e.g., ``0.0.0.0:8080->80/tcp``) for running containers.
- **Pods**: Port mappings from pod member containers are aggregated and shown in the pod details view.

This makes it easy to verify which services are exposed and on which host ports, without leaving the terminal.

Config Inspection
-----------------
Press ``g`` on any selected resource (container, pod, image, volume, or network) to inspect its full configuration. This runs the engine's inspect command and displays the raw JSON output in a scrollable popup.

- **Containers**: Shows the output of ``docker inspect`` or ``podman inspect``.
- **Pods**: Shows the output of ``podman pod inspect``.
- **Images, Volumes, Networks**: Shows the corresponding inspect output for the selected resource.

This is useful for debugging configuration issues, verifying environment variables, checking mount points, and inspecting network settings without switching to a separate terminal.

Image Management
----------------
- **Image Listing**: Browse through downloaded container images effortlessly.
- **Direct Pulling**: Pull images directly using standard image names/tags via a dedicated UI modal.
- **Registry Search**: Interactively search online registries, browse detailed results including descriptions and star ratings, and pull selected options.
- **Configure Registries**: Directly modify and configure local unqualified search registries (especially useful for Podman users).

Interactive Logs & Shell Execution
----------------------------------
- **Embedded Logs**: View a live, scrollable embedded log viewer for running or stopped containers. Navigate log lines individually and copy selected log lines directly to your system clipboard (using ``y`` or ``c``).
- **Custom Exec Commands**: Spawn a temporary subprocess to execute any custom command inside a running container.
- **Interactive Shell**: Drop immediately into a ``/bin/sh`` or ``/bin/bash`` shell on a running container without breaking your flow.

Volumes and Networks
--------------------
Browse active volumes and network definitions. Removing dangling or unused networks and volumes directly from the interface helps maintain a clean host environment.
