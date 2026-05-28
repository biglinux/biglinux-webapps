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

- Built with Rust using GTK4 and libadwaita
- WebKitGTK 6.0 for webapp viewer with isolated profiles
- Uses website scraping to extract icons and metadata
- Integrated with desktop environment via desktop files
- Compatible with both Xorg and Wayland display servers

## Screenshots

![webapps07](https://github.com/user-attachments/assets/58e75c37-e93a-4b5f-a696-7990bf005286)

![webapps08](https://github.com/user-attachments/assets/00aff0ad-7b3c-49ff-a363-9ef76f3ef233)

![webapps05](https://github.com/user-attachments/assets/b2a23dfe-e761-43d3-87cf-78d3aeea939a)

![webapps06](https://github.com/user-attachments/assets/7c6759c9-3abd-465a-92b3-53bb71450f36)


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

- gtk4 (>= 4.10)
- libadwaita-1 (>= 1.6)
- webkitgtk-6.0 (>= 2.50)
- gettext
- openssl

## Development Validation

Run the repo-local validator before sending changes for review:

```bash
bash scripts/validate-customizations.sh
```

## Contributing Translations

Translations use gettext. Source catalogs live in `po/`.

**Extracting new strings** (after adding `gettext("…")` calls in Rust source):

```bash
# update the .pot template and merge it into every catalog
./scripts/update-translations.sh
```

**Adding a new language** (`xx` = locale code):

```bash
cp po/en.po po/xx.po
# edit xx.po with your translations
```

**Building .mo files** (done automatically by PKGBUILD):

```bash
for po in po/*.po; do
    lang=$(basename "$po" .po)
    msgfmt -o "po/${lang}.mo" "$po"
done
```

Runtime `.mo` files are installed to `/usr/share/locale/<lang>/LC_MESSAGES/biglinux-webapps.mo`.
