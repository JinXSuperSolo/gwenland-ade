import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";

// design system is class-based (.dark); follow the OS theme
const media = window.matchMedia("(prefers-color-scheme: dark)");
const syncTheme = (dark: boolean) =>
  document.documentElement.classList.toggle("dark", dark);
syncTheme(media.matches);
media.addEventListener("change", (e) => syncTheme(e.matches));

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
