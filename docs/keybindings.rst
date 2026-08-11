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
     - Cycle through all UI panes (Running -> Stopped -> Images -> Volumes -> Networks -> Pods -> Logs).
   * - ``BackTab``
     - Cycle through UI panes in reverse order.
   * - ``?``, ``Esc``
     - Toggle the global Help popup tooltip on and off.
   * - ``E``
     - Toggle the engine filter (Docker only, Podman only, or Both).
   * - ``r``
     - Refresh all data from the container engines.
   * - ``g``
     - Inspect the selected resource (container, pod, image, volume, or network). Displays raw JSON output.

Navigation
----------
Navigate through lists of containers, images, volumes, networks, and pods using these keys:

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
     - Execute the primary action for the selected item (e.g., view logs for containers, create container from an image).

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
   * - ``S``, ``u``
     - Start the selected stopped container (alias for ``s`` on the Stopped tab).
   * - ``d``, ``Delete``
     - Prompt to remove the selected resource (container, image, volume, network, or pod).
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
   * - ``P``
     - Create a new pod with a custom name (Pods tab).

Logs Panel Bindings
-------------------
When the logs panel is focused (e.g., by pressing ``Right`` or ``Enter`` while on a container tab), the following bindings are active:

.. list-table:: Logs Panel Keybindings
   :widths: 20 80
   :header-rows: 1

   * - Key
     - Action
   * - ``Up``, ``k``
     - Scroll up through log lines.
   * - ``Down``, ``j``
     - Scroll down through log lines.
   * - ``y``, ``c``
     - Copy the currently selected log line to the system clipboard.
   * - ``Esc``, ``Left``, ``h``
     - Exit the logs focus mode and return to container selection.
   * - ``x``, ``e``, ``i``
     - Open interactive exec/shell prompt for the container.
   * - ``q``
     - Exit the logs focus mode and return to container selection.

Mouse Bindings
--------------
Lazypod supports mouse interaction for navigating the interface:

.. list-table:: Mouse Bindings
   :widths: 20 80
   :header-rows: 1

   * - Action
     - Effect
   * - **Scroll Up**
     - Scroll up in the active list or log viewer.
   * - **Scroll Down**
     - Scroll down in the active list or log viewer.
   * - **Left Click** (left panel)
     - Select a panel (Running, Stopped, Images, Volumes, Networks) and select the clicked item.
   * - **Left Click** (right panel)
     - Focus the logs/detail panel when on a container tab.
