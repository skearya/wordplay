import { RouteDefinition } from "@solidjs/router";
import { lazy } from "solid-js";

export const routes: RouteDefinition[] = [
  {
    path: "/",
    component: lazy(() => import("./pages/Home")),
  },
  {
    path: "/room/:name",
    component: lazy(() => import("./pages/Game")),
  },
  {
    path: "/choose-username",
    component: lazy(() => import("./pages/ChooseUsername")),
  },
  {
    path: "**",
    component: lazy(() => import("./pages/404")),
  },
];
