// First-time onboarding state (runes module) — GWEN-490.
//
// Not a modal: the onboarding *is* the composer's empty state, with a guided
// flow. `isFirstTime` is set from the `has_memory` command on app mount.

export const onboarding = $state({
  // No memory on disk yet → this is a fresh install.
  isFirstTime: false,
  // Set once the user has picked a workspace during onboarding.
  workspaceChosen: false,
  // The one-time "you can detach the preview" hint, shown after the first
  // generate and auto-dismissed.
  showDetachHint: false,
});

let hintTimer: ReturnType<typeof setTimeout> | undefined;

/// Called once the user picks a workspace: advances the flow.
export function dismissWorkspacePrompt() {
  onboarding.workspaceChosen = true;
}

/// Shows the "Tip: detach preview" hint and auto-dismisses it after 5s
/// (GWEN-490). Safe to call repeatedly — it only fires for first-timers and
/// only once per session.
let hintShownThisSession = false;
export function maybeShowDetachHint() {
  if (!onboarding.isFirstTime || hintShownThisSession) return;
  hintShownThisSession = true;
  onboarding.showDetachHint = true;
  hintTimer = setTimeout(() => (onboarding.showDetachHint = false), 5000);
}

/// Dismiss the hint early (e.g. the user started typing).
export function dismissDetachHint() {
  onboarding.showDetachHint = false;
  if (hintTimer) clearTimeout(hintTimer);
}
