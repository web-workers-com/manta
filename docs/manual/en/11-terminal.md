# Terminal {#terminal}

Below each pane you can show a terminal that follows the pane's folder – and the other way round. It runs where the pane is: on your Mac, on an SFTP server or in a container.

![A pane with a terminal below](images/terminal.png)

## Show the terminal {#einblenden}

- Press **⌃`** (**Toggle Terminal**).
- Or choose **Toggle Terminal** in the command palette.

The terminal appears below the active pane. Drag its top edge to change its height.

More commands in the command palette:

- **New Terminal Session** – starts a fresh shell.
- **Clear Terminal** – clears the display.

If you end the shell (with `exit`, for example), the terminal shows "[Process exited — press Enter to restart]". If the pane shows a server or share (SFTP, SMB, Docker, Kubernetes), `exit` leaves it there too: the pane goes back to the folder on your Mac it showed before, and the connection closes unless another tab still shows it. If the shell breaks off instead (connection lost, container stopped), the pane stays where it is.

## Pane and terminal follow each other {#folgen}

**From pane to terminal:** when you change folders in the pane, the shell `cd`s there – as soon as it's idle, that is, when no command is running. Until then a note says "Will cd to “folder” when idle". Anything you've half typed is kept.

**From terminal to pane:** type `cd ~/Projects` in the terminal and the pane above shows that folder. Turn this off with **Pane follows terminal cd** (see [Settings](#einstellungen)).

Your prompt and shell configuration stay as they are.

## Terminal in containers and on servers {#container-ssh}

The terminal runs where the pane is:

| The pane shows … | Terminal |
|---|---|
| a local folder (cloud folders too) | your normal shell on the Mac |
| a Docker container | a shell inside the container (`docker exec`) |
| a Kubernetes pod | a shell inside the pod (`kubectl exec`) |
| an SFTP server | `ssh` to the server, with your SSH config and ssh-agent |
| S3, FTP, an archive | no terminal ("No terminal for this location") |

- Manta installs nothing in the container or on the server for this.
- For SFTP connections with a password, you type the password in the terminal. A key file that isn't in your SSH config or agent asks in the terminal too.
- Each pane keeps up to four shells (local, containers, servers). When you go back, the earlier shell is there again with its history.
- If a container is stopped, the terminal says "Container “name” is stopped". Start it and click the note.

Right-click → **Open Shell in Container** starts a shell in the container folder you're looking at.

### `ssh` in the local terminal {#ssh-erkennen}

If you type `ssh` yourself in the local terminal (e.g. `ssh anna@server` or a host from your `~/.ssh/config`), Manta notices. While the connection runs, **Open anna@server in the pane** appears in the terminal's top-right corner. Click it to show the server in the pane over SFTP: Manta reuses an open connection to it, else a matching favorite, else the Connect dialog opens filled in (you only enter the password). From then on pane and terminal follow each other as with a container, and you can copy and move between the panes.

- The terminal then starts its own shell on the server; with password login it asks for the password there once more. The `ssh` session you typed stays in the background until you go back to the local folder.
- `ssh` is noticed in zsh (the macOS default shell), also through an alias. `ssh` with a command after it (`ssh server uptime`) or just for a tunnel (`-N`) offers nothing.

## File actions in the terminal {#dateiaktionen}

When you copy, move, delete or rename something in Manta, the terminal shows the matching shell command – such as `cp -R …`, `scp …`, `docker cp …` or `aws s3 cp …` – followed by the result ("done", "failed: …"). Moving to the Trash shows up as `mv … ~/.Trash/`, a permanent delete as `rm -r …`.

> [!NOTE]
> This is only a display. Manta does the work itself; nothing is run in the shell. You can copy the commands, though, for your own scripts for example.

On remote shells (containers, SSH) Manta shows these commands as a card on the terminal instead of writing them into the shell.

To turn it off: **Show file actions in terminal** in the settings.

## Settings {#einstellungen}

Open **Settings** (**⌘,**) → **Plugins** → **Terminal** → **Settings**.

| Setting | Meaning |
|---|---|
| **Shell** | Which shell starts (`/bin/zsh`, for example). |
| **Font size** | Size of the terminal font. |
| **Scrollback** | How many lines you can scroll back. |
| **Use Option as Meta key** | Option+key sends ESC+key (for Emacs-style shortcuts) instead of typing special characters. |
| **Pane follows terminal cd** | A `cd` in the terminal changes the pane's folder. |
| **Show file actions in terminal** | Shows file actions as shell commands (see above). |

> [!TIP]
> On many non-US keyboards you need Option for `@`, `{`, `[` and `|`. Leave **Use Option as Meta key** off unless you use Emacs-style shortcuts.

## Keyboard in the terminal {#tastatur}

While the terminal has focus, these shortcuts belong to the terminal instead of Manta: **⌘C**, **⌘V**, **⌘A**, **⌘K**, **⌘←**, **⌘→**, **⌘⌫** and **⌘Z**. Other Manta shortcuts such as **⇧⌘P** keep working. Click into the file list to give the keyboard back to Manta.

## Limitations {#grenzen}

- busybox `sh` (Alpine) and bash 3 can't keep half-typed input when the terminal follows the pane (bash 4 or later and zsh can).
- In containers with a shell other than bash or POSIX `sh`, the terminal follows only one way.
- No terminal for S3, FTP and archives.
