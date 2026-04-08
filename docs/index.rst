Welcome to Lazypod's documentation!
===================================

.. toctree::
   :maxdepth: 2
   :caption: Contents:

Lazypod is a modern, responsive Terminal User Interface (TUI) for container management, supporting both **Docker** and **Podman**. It is inspired by the interface of `lazydocker <https://github.com/jesseduffield/lazydocker>`_.

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

Contributing
------------

We welcome contributions! Please see our ``CONTRIBUTING.md`` guide located in the repository root for more details on local development, testing, and our required branching strategy.

License
-------

This project is licensed under the MIT License, an `OSI-approved <https://opensource.org/licenses/MIT>`_ open-source license. See the ``LICENSE`` file for details.

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
