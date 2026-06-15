Installation
============

Prerequisites
-------------
Before installing Lazypod, ensure you have the following installed on your system:

- **Rust and Cargo**: The Rust toolchain is required to build the application from source. You can install it via `rustup <https://rustup.rs/>`_. The Minimum Supported Rust Version (MSRV) is 1.75.
- **Docker or Podman**: At least one of these container engines must be installed and running. If both are installed, Lazypod can manage both simultaneously.

Building from Source
--------------------

Currently, Lazypod is available by building it from source. Follow these steps:

1. Clone the repository:

.. code-block:: bash

    git clone https://github.com/yourusername/lazypod.git
    cd lazypod

2. Build and run the application:

.. code-block:: bash

    cargo run

3. To install it globally on your system, you can use:

.. code-block:: bash

    cargo install --path .

This will place the ``lazypod`` executable in your Cargo bin directory (usually ``~/.cargo/bin``), which should be added to your system's PATH.

Permissions
-----------
Depending on your system's configuration for Docker and Podman, you may need to run Lazypod with appropriate permissions.
- For Docker, ensure your user is part of the ``docker`` group, or you may encounter permission denied errors when Lazypod attempts to fetch container data.
- Podman is often configured to run rootless, which typically requires no extra permissions.