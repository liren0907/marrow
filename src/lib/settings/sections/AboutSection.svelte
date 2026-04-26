<script lang="ts">
  // Version intentionally hard-coded — wire up against package.json later
  // if we want a single source of truth.
  const VERSION = "0.1.0";
  const REPO_URL = "https://github.com/liren0907/marrow";
</script>

<div class="about">
  <div class="brand">
    <div class="logo" aria-hidden="true">🦴</div>
    <div>
      <div class="name">Marrow</div>
      <div class="version">v{VERSION}</div>
    </div>
  </div>

  <p class="tagline">
    Local-first, multi-modal note-taking. Your folder is your workspace; your
    files stay files.
  </p>

  <div class="links">
    <a href={REPO_URL} target="_blank" rel="noreferrer noopener">
      GitHub repository →
    </a>
  </div>

  <div class="shortcuts">
    <h3 class="shortcuts-title">Keyboard shortcuts</h3>
    <dl>
      <dt>⌘P</dt><dd>Quick open / full-text search</dd>
      <dt>⇧⌘P</dt><dd>Command palette</dd>
      <dt>⌘J</dt><dd>Toggle bottom panel</dd>
      <dt>⌘\</dt><dd>Split pane</dd>
      <dt>⌘1 / ⌘2</dt><dd>Focus pane by index</dd>
      <dt>⇧⌘] / ⇧⌘[</dt><dd>Next / previous tab</dd>
      <dt>⌘S / ⌘W</dt><dd>Save / close active tab</dd>
      <dt>⌘,</dt><dd>Quick tweaks (theme + accent)</dd>
      <dt>⇧⌘,</dt><dd>Open this Settings page</dd>
    </dl>
  </div>
</div>

<style>
  .about {
    color: var(--color-base-content);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
  }
  .logo {
    font-size: 40px;
    line-height: 1;
  }
  .name {
    font-size: 18px;
    font-weight: 600;
  }
  .version {
    font-size: 11px;
    color: var(--mw-ink-3);
    font-family: var(--font-mono);
  }
  .tagline {
    font-size: 13px;
    color: var(--mw-ink-2);
    line-height: 1.6;
    margin: 0 0 16px;
    max-width: 420px;
  }
  .links {
    margin-bottom: 24px;
  }
  .links a {
    font-size: 13px;
    color: var(--mw-accent);
    text-decoration: none;
  }
  .links a:hover {
    text-decoration: underline;
  }
  .shortcuts-title {
    font-size: 13px;
    font-weight: 600;
    margin: 0 0 8px;
  }
  dl {
    display: grid;
    grid-template-columns: 110px 1fr;
    gap: 4px 16px;
    margin: 0;
    font-size: 12px;
  }
  dt {
    font-family: var(--font-mono);
    color: var(--mw-ink-2);
  }
  dd {
    margin: 0;
    color: var(--color-base-content);
  }
</style>
