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

![webapps01](https://github.com/user-attachments/assets/e976037d-228d-4217-be25-4ef3926f916b)

![webapps02](https://github.com/user-attachments/assets/37d66ea6-aca7-4a92-a0e2-8122bca4dcd7)

![webapps03](https://github.com/user-attachments/assets/874e8265-1154-441c-9a61-06fd188ef7f3)

![webapps04](https://github.com/user-attachments/assets/e49e0bf9-38cc-479d-ad58-0e2f143c2d1f)


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
