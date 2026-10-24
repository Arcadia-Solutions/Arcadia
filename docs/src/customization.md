# Customization & Theming

Arcadia allows administrators to personalize the visual branding, landing pages, stylesheets, and assets of their instance.

## Custom Landing Page

By default, visiting the root URL of an Arcadia instance displays the login view. If you wish to present a custom HTML landing page to unauthenticated visitors:

1. In `config.yml`, set:
   ```yaml
   frontend:
     enable_custom_front_page: true
   ```
2. Place your custom HTML file at `public/home/index.html` (inside the frontend directory or container).
3. When enabled, non-logged-in visitors landing on `/` are served this page directly (none of Arcadia's files are loaded), while authenticated users get the home page (`HomeView.vue`).

The page is served at `/`, so links to its own assets must be absolute (e.g. `/home/style.css`, not `style.css`).

> [!NOTE]
> The bundled Caddy server tells both apart with the `arcadia_logged_in` cookie, set by the frontend once a user is signed in. If you serve the frontend with another web server, serve `home/index.html` for requests to `/` that don't carry the `arcadia_logged_in=1` cookie (see `frontend/Caddyfile`). The cookie is set through the Cookie Store API, which browsers only expose over HTTPS (or `localhost`), so the custom front page requires HTTPS.

## Unauthenticated Pages (Custom CSS & JS)

Pages accessible before signing in (`/login`, `/register`, `/apply`, `/reset-password`) cannot use the user-selectable CSS stylesheets configured in the database, because no user session exists. 

To apply custom branding to these pages, create two optional, git-ignored files:

- `frontend/public/custom_unauth.css`: Custom CSS rules loaded on unauthenticated pages.
- `frontend/public/custom_unauth.js`: Custom JavaScript executed on unauthenticated pages.

> [!NOTE]
> Creating or editing these files requires updating the frontend (`docker compose up -d frontend --build`) 

## Site Assets & Branding

You can replace the default placeholder graphics with your own site branding by replacing the following asset files:

| Asset | File Location | Purpose |
| :--- | :--- | :--- |
| **Site Logo** | `frontend/src/assets/logo.svg` | Main navbar logo (falls back to `logo.example.svg` if omitted) |
| **Favicon** | `frontend/public/favicon.ico` | Browser tab icon |
| **Default Avatar** | `frontend/public/default_user_avatar.png` | Fallback avatar for users who haven't uploaded one |

## Custom Icons (SVG Overrides)

Arcadia allows overriding any PrimeIcon across the frontend with a custom SVG file.

To replace an icon, place your `.svg` file into `frontend/src/assets/custom-icons/` matching the PrimeIcon name without the `pi-` prefix:

- For example, to override the Bonus Points icon (`pi-wallet`), save your SVG as:
  ```
  frontend/src/assets/custom-icons/wallet.svg
  ```
- The build automatically generates CSS mask rules that replace the icon font glyph across the entire application with your SVG, seamlessly preserving theme colors (`currentColor`), sizes, and hover effects.

## Rebuilding

Custom icons and assets are inlined into the compiled frontend bundle. After adding or changing assets in `frontend/src/assets/custom-icons/`, rebuild the frontend. For example, with docker:

```bash
docker compose build frontend
docker compose up -d frontend
```

## User Stylesheets (CSS Sheets)

Once users are logged in, Arcadia supports custom theme stylesheets. Users with the `create_css_sheet` and `edit_css_sheet` permissions can add and manage site-wide stylesheets through the Staff Dashboard:

- Each stylesheet is stored in the database (`css_sheets` table) with a unique name, CSS content, and an optional preview image.
- Users can select their preferred active theme in their account settings.
- The instance's default theme is defined in the site settings (`default_css_sheet_name`, defaults to `arcadia`).
