import { RouteDefinition } from "@solidjs/router";
import { lazy } from "solid-js";

export const routes: RouteDefinition[] = [
  {
    path: "/",
    component: lazy(() => import("./pages/home")),
  },
  {
    path: "/room/:name",
    component: lazy(() => import("./pages/game")),
  },
  {
    path: "**",
    component: lazy(() => import("./pages/404")),
  },
];
