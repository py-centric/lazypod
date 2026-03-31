Welcome to Lazypod's documentation!
===================================

.. toctree::
   :maxdepth: 2
   :caption: Contents:

Lazypod is a modern, responsive Terminal User Interface (TUI) for container management, supporting both **Docker** and **Podman**.

Features
--------

- Complete interactive TUI for Docker and Podman via ``std::process::Command`` integrations.
- Multi-engine support: toggle between viewing Docker containers, Podman pods, or both.
- View, start, stop, and remove running and stopped containers.
- View images, pull from registry via interactive search, and spawn containers from images.
- Direct Image Pulling and interactive configuration of Podman search registries.
- Run Container popup with support for passing environment variables.
- View volumes and networks.
- Direct interactive shell dropping into running containers (``/bin/sh`` or custom command).
- Embedded real-time log viewer for pods.
- Contextual Help Bar depending on active tab and global Help Tooltips for keybindings.

Installation & Usage
--------------------

Simply run from the root directory:

.. code-block:: bash

   cargo run

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
