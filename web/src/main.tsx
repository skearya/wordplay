import { Index } from "./pages/Index";
import "./root.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Route, Switch } from "wouter";

createRoot(document.getElementById("root")!).render(
	<>
		<Switch>
			<Route path="/" component={Index} />
			<Route>404</Route>
		</Switch>
	</>,
);
