# BigLinux WebApps

A modern GTK4 tool to create and manage webapps, supporting multiple browsers while detecting icons and titles automatically. Integrate your favorite web applications into your desktop environment.

## Features

- **Multi-browser Support**: Create webapps using any installed browser
- **Automatic Detection**: Automatically detects website titles and favicons
- **Categorization**: Organize webapps by categories
- **Search Functionality**: Quickly find your webapps using the search feature
- **Import/Export**: Easily backup and restore your webapps collection
- **Browser Switching**: Change browsers for existing webapps without recreating them
- **Customized Profiles**: Uses optimized browser profiles for a better webapp experience

## Technical Details

- Built with Python using GTK4 and libadwaita
- Uses website scraping to extract icons and metadata
- Integrated with desktop environment via desktop files
- Compatible with both Xorg and Wayland display servers

## Screenshots

![WebApps Manager Main Window](https://github.com/biglinux/biglinux-webapps/assets/25956396/bf5d545f-4f86-452e-a470-5812f34f77c9)
![WebApp Creation Dialog](https://github.com/biglinux/biglinux-webapps/assets/25956396/de66d6de-0247-4a28-97c0-4ec7674c9321)

## Installation

The package is available in BigLinux repositories:

```bash
sudo pacman -S biglinux-webapps
```

## Usage

1. Launch the application from your menu or run:
   ```bash
   big-webapps-gui
   ```
2. Click the "Add" button to create a new webapp
3. Enter the URL, name, and select a browser
4. Enjoy your new integrated webapp!

## License

GPL-3.0

## Dependencies

- python-bs4
- python-requests
- gettext
- python-pillow
- python-gobject
