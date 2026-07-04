// Shared UI state across shell components (runes module)

// Settings sections. `null` = the card-grid launcher; a value deep-links
// straight into that section (e.g. Ctrl+M → "memory").
export type SettingsSection = null | "api-keys" | "memory" | "about";

export const ui = $state({
  sidebarPinned: false,
  settingsOpen: false,
  // Which settings section is showing (null = card grid).
  settingsSection: null as SettingsSection,
  // Preview pane popped out into its own Tauri window (GWEN-489).
  previewDetached: false,
  // Preview pane hidden by default; it expands when output/tools need it or the
  // user toggles it on. Auto-revealed on the first generate.
  previewVisible: false,
});

/// Opens Settings, optionally deep-linked to a section.
export function openSettings(section: SettingsSection = null) {
  ui.settingsSection = section;
  ui.settingsOpen = true;
}
