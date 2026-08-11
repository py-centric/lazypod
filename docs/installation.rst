Installation
============

Prerequisites
-------------
Before installing Lazypod, ensure you have the following installed on your system:

- **Rust and Cargo**: The Rust toolchain is required to build the application from source. You can install it via `rustup <https://rustup.rs/>`_. The Minimum Supported Rust Version (MSRV) is 1.75.
- **Docker or Podman**: At least one of these container engines must be installed and running. If both are installed, Lazypod can manage both simultaneously.

Building from Source
--------------------

1. Clone the repository:

.. code-block:: bash

    git clone https://github.com/py-centric/lazypod.git
    cd lazypod

2. Build and run the application:

.. code-block:: bash

    cargo run

3. To build an optimized release binary:

.. code-block:: bash

    cargo build --release

The binary will be at ``target/release/lazypod``.

Installing with Cargo
---------------------

You can install Lazypod globally using Cargo's install command. This compiles the binary and places it in your Cargo bin directory (usually ``~/.cargo/bin``):

.. code-block:: bash

    cargo install --path .

Make sure ``~/.cargo/bin`` is on your system's ``PATH``. After installation, you can run ``lazypod`` from anywhere.

Permissions
-----------
Depending on your system's configuration for Docker and Podman, you may need to run Lazypod with appropriate permissions.
- For Docker, ensure your user is part of the ``docker`` group, or you may encounter permission denied errors when Lazypod attempts to fetch container data.
- Podman is often configured to run rootless, which typically requires no extra permissions.

License
-------
Lazypod is licensed under the `GNU Affero General Public License v3.0 <https://www.gnu.org/licenses/agpl-3.0.html>`_ (AGPL-3.0). A separate commercial license is available for organizations that cannot comply with the AGPL-3.0 terms. See the ``LICENSE`` file in the repository root for details.
