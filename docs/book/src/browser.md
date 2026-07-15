# Browser verification

Pleiades can run optional Playwright checks from the live workspace.

Commands:

```text
/browser open <url>
/browser screenshot
/browser inspect
/browser console
/browser close
```

Headless commands exist for discoverability:

```bash
pleiades browser open <url>
pleiades browser screenshot
pleiades browser inspect
pleiades browser console
pleiades browser close
```

The live workspace owns browser session state. Headless commands print guidance
instead of pretending to keep a browser alive after the process exits.

## Setup

Pleiades does not bundle Node.js, Playwright, or browser binaries. Install them
in the project or make them resolvable from the workspace:

```bash
npm install -D playwright
npx playwright install chromium
```

## Behavior

- `/browser open <url>` launches Chromium headlessly through Playwright,
  navigates to the URL, records status/title/HTML size, captures console
  messages, records failed requests, and stores the report in the runtime.
- `/browser screenshot` revisits the last opened URL and writes a PNG below
  `.pleiades/browser/screenshots/`.
- `/browser inspect` renders the last report.
- `/browser console` renders console messages from the last report.
- `/browser close` clears the stored browser report.

## Limits

- Browsers are launched per command in this slice; there is no persistent page
  object yet.
- DOM assertions, accessibility inspection, form interaction, and visual
  diffing are planned follow-up work.
- Playwright failures are reported as command errors with setup guidance.
