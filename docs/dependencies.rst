Dependencies & Licensing
========================

This inventory details all production and development dependencies utilized by Lazypod, their associated OSI open source licenses, and their commercialization considerations under the project's dual AGPLv3/Commercial licensing model.

Core Dependencies
-----------------

.. list-table::
   :widths: 25 15 35 25
   :header-rows: 1

   * - Crate
     - Version
     - License
     - Purpose
   * - ``ratatui``
     - ``0.26``
     - MIT
     - Terminal user interface layout, widgets, and rendering pipeline.
   * - ``crossterm``
     - ``0.27``
     - MIT
     - Cross-platform terminal manipulation and asynchronous raw input polling.
   * - ``tokio``
     - ``1.36``
     - MIT
     - Asynchronous runtime, task spawning, and thread event synchronization.
   * - ``serde`` / ``serde_json``
     - ``1.0``
     - MIT / Apache-2.0
     - Serialization and schema-neutral deserialization of container engine JSON outputs.
   * - ``chrono``
     - ``0.4``
     - MIT / Apache-2.0
     - Timestamp parsing and locale-agnostic humanized date formatting.
   * - ``clap``
     - ``4.5``
     - MIT / Apache-2.0
     - Command-line argument parsing and flag evaluation.
   * - ``arboard``
     - ``3.6.1``
     - MIT / Apache-2.0
     - System clipboard integration for log extraction.
   * - ``async-trait``
     - ``0.1.89``
     - MIT / Apache-2.0
     - Dynamic dispatch for asynchronous trait definitions (``EngineClient``).
   * - ``shlex``
     - ``1.3``
     - MIT / Apache-2.0
     - POSIX-compliant shell argument splitting for container run and exec commands.
   * - ``thiserror``
     - ``1.0``
     - MIT / Apache-2.0
     - Structured domain error types.
   * - ``tracing`` / ``tracing-subscriber``
     - ``0.1`` / ``0.3``
     - MIT
     - Structured diagnostic logging and runtime observability.

Development & Test Dependencies
-------------------------------

.. list-table::
   :widths: 25 15 35 25
   :header-rows: 1

   * - Crate
     - Version
     - License
     - Purpose
   * - ``rstest``
     - ``0.24``
     - MIT / Apache-2.0
     - Fixture-based and parameterized test execution.

Commercialization & Distribution Considerations
-----------------------------------------------

1. **Permissive Ingestion**: All direct dependencies are licensed under permissive open-source licenses (MIT or dual MIT/Apache-2.0). There are zero GPL/LGPL/copyleft dependencies linked into the Lazypod binary.
2. **Proprietary Embeddability**: Because all upstream libraries are MIT/Apache-2.0, PyCentric possesses unencumbered authority to grant commercial, proprietary non-AGPL licenses to enterprise customers.
3. **No Network Leakage**: The application communicates exclusively via local CLI processes (``docker`` / ``podman``) and does not bundle or link against engine daemon internals.
