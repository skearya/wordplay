/* @refresh reload */
import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import { Router } from "@solidjs/router";
import { render } from "solid-js/web";
import "./index.css";
import { routes } from "./routes";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?",
  );
}

render(() => <Router>{routes}</Router>, root!);
