// Artifact preview state (runes module).
//
// The agent calls `GL_OpenPreview` to show a rendered artifact in the preview
// pane — HTML, Markdown, a Mermaid diagram, or plain code. This holds the
// current artifact and opens the pane; the pane renders it (see PreviewPane).

import { ui } from "../../shared/ui.svelte";

export type ArtifactKind = "html" | "markdown" | "mermaid" | "code";

export type Artifact = {
  title: string;
  kind: ArtifactKind;
  content: string;
  /// For `code`, an optional language label.
  language?: string;
};

export const artifact = $state({
  current: null as Artifact | null,
});

/// Opens the preview pane showing the given artifact.
export function openArtifact(a: Artifact) {
  artifact.current = a;
  ui.previewVisible = true;
}

/// Closes the artifact preview pane.
export function closeArtifact() {
  ui.previewVisible = false;
}
