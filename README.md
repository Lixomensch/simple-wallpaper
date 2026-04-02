# simple-wallpaper (`swp`)

A minimal CLI wallpaper manager for Arch Linux (and other Linux desktops).

## Supported desktop environments

| Desktop / WM  | Backend tool                  | Install (Arch)              |
|---------------|-------------------------------|-----------------------------|
| KDE Plasma    | `plasma-apply-wallpaperimage` | `plasma-workspace` (built-in) |
| GNOME / Cinnamon / Budgie | `gsettings`      | built-in                    |
| Hyprland      | `swww`                        | `sudo pacman -S swww`       |
| Sway          | `swaybg`                      | `sudo pacman -S swaybg`     |
| X11 (any WM)  | `feh`                         | `sudo pacman -S feh`        |

The correct backend is chosen automatically at runtime via `XDG_CURRENT_DESKTOP`.

## Installation

```bash
git clone https://github.com/youruser/simple-wallpaper
cd simple-wallpaper
cargo install --path .
```

The binary is installed as `swp`.

## Wallpaper directory

Managed wallpapers live in:

```
~/.local/share/swp/wallpapers/
```

Use `swp add` to import images there (subdirectories are supported).
Run `swp path` to print the full path.

Supported formats: `jpg`, `jpeg`, `png`, `webp`, `bmp`, `avif`.

## Usage

```
swp <COMMAND>
```

### set — apply a specific wallpaper

```bash
swp set floresta.jpg          # lookup by filename in the managed directory
swp set /home/user/bg.png     # apply any image by absolute path
```

### add — import image files into the managed directory

```bash
swp add /home/user/Pictures/bg1.jpg
swp add ~/Pictures/bg1.jpg ~/Pictures/bg2.png
```

If a filename already exists, `swp` keeps both files by appending a numeric
suffix (for example `bg1_1.jpg`).

### random — apply a random wallpaper

```bash
swp random
```

### lists — manage wallpaper collections

```bash
swp lists
```

Shows all available lists.

#### lists create

```bash
swp lists create my-favorites
```

Creates a new list.

#### lists delete

```bash
swp lists delete my-favorites
```

Deletes a list.

#### lists show

```bash
swp lists show my-favorites
swp lists show my-favorites --plain
```

Shows wallpapers in a list.

#### lists add

```bash
swp lists add my-favorites floresta.jpg praia.png
```

Adds wallpapers to a list.

#### lists remove

```bash
swp lists remove my-favorites floresta.jpg
```

Removes wallpapers from a list by name or UUID.

#### lists play — start wallpaper rotation in the background

```bash
swp lists play my-favorites          # play from a specific list
swp lists play my-favorites --interval 30s
swp lists play --interval 5m         # no list: direct filesystem mode
swp lists play --interval 1h
```

The command returns immediately and keeps running in the background.
Playback state is saved to `~/.config/swp/playback.json`.

#### lists stop

```bash
swp lists stop
```

Stops the active playback process and clears the persisted state.

### wallpapers — show all available wallpapers

```bash
swp wallpapers
```

### path — print the wallpaper directory

```bash
swp path
# → /home/user/.local/share/swp/wallpapers
```

## Auto-start on login

Playback can be resumed automatically in a desktop session with a systemd user
service. The service should call the hidden internal resume path and rely on
`~/.config/swp/playback.json` as the source of truth.

An example unit is provided at [systemd/swp-resume.service](systemd/swp-resume.service).
It is intentionally `oneshot` and should be enabled under
`graphical-session.target`.

Recommended setup:

```bash
mkdir -p ~/.config/systemd/user
cp systemd/swp-resume.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now swp-resume.service
```

When playback is not active, or the persisted state is missing/corrupt, the
resume path exits without blocking the session.

## Notes for specific desktops

**Hyprland / generic Wayland:** start `swww-daemon` before using `swp`.
You can add it to your Hyprland config:

```
exec-once = swww-daemon
```

**Sway:** `swp` spawns `swaybg` in the background and kills the previous
instance automatically. No extra setup needed.

**X11:** `feh --bg-fill` is used. `feh` writes `~/.fehbg` so the wallpaper
survives across sessions when sourced from your WM startup script.
