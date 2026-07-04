import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import PreviewWindow from "./PreviewWindow.svelte";

// design system is class-based (.dark); follow the OS theme
const media = window.matchMedia("(prefers-color-scheme: dark)");
const syncTheme = (dark: boolean) =>
  document.documentElement.classList.toggle("dark", dark);
syncTheme(media.matches);
media.addEventListener("change", (e) => syncTheme(e.matches));

// The detached preview window loads the same bundle with `?preview`, and mounts
// a preview-only root instead of the full app shell (GWEN-489).
const isPreview = new URLSearchParams(window.location.search).has("preview");

const app = mount(isPreview ? PreviewWindow : App, {
  target: document.getElementById("app")!,
});

export default app;
