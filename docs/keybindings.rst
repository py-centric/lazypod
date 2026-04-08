Keybindings
===========

Lazypod's keybindings are designed to be fast and intuitive, drawing inspiration from tools like Vim and other terminal-based interfaces.

Global Bindings
---------------
The following keybindings are available globally across the application, regardless of the active tab.

.. list-table:: Global Keybindings
   :widths: 20 80
   :header-rows: 1

   * - Key
     - Action
   * - ``q``, ``Ctrl+C``
     - Quit the application.
   * - ``Tab``
     - Cycle through all UI panes (Running -> Stopped -> Images -> Volumes -> Networks -> Logs).
   * - ``?``, ``Esc``
     - Toggle the global Help popup tooltip on and off.
   * - ``E``
     - Toggle the engine filter (Docker only, Podman only, or Both).

Navigation
----------
Navigate through lists of containers, images, volumes, and networks using these keys:

.. list-table:: Navigation Keybindings
   :widths: 20 80
   :header-rows: 1

   * - Key
     - Action
   * - ``Up``, ``k``
     - Move the selection up one item. Switches to the previous pane at the top boundary.
   * - ``Down``, ``j``
     - Move the selection down one item. Switches to the next pane at the bottom boundary.
   * - ``Enter``
     - Execute the primary action for the selected item (e.g., Run a container from an image).

Context-Specific Actions
------------------------
Depending on the active tab and selected item, these bindings execute specific container engine operations.

.. list-table:: Action Keybindings
   :widths: 20 80
   :header-rows: 1

   * - Key
     - Action (Context)
   * - ``s``
     - Toggle container state: Stop (if in Running tab) or Start (if in Stopped tab).
   * - ``d``, ``Delete``
     - Prompt to remove the selected resource (container, image, volume, or network).
   * - ``i``, ``e``
     - Open an interactive shell (``/bin/sh``) inside the selected running container.
   * - ``x``
     - Open a prompt to input a custom command to execute inside the running container.
   * - ``/``
     - Open the interactive search prompt to find images online (Images tab).
   * - ``p``
     - Open a prompt to pull a specific image directly by name/tag (Images tab).
   * - ``c``
     - Configure local unqualified search registries (Images tab).
