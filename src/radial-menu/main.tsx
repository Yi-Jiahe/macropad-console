import React from "react";
import ReactDOM from "react-dom/client";
import { RadialMenu } from "./RadialMenu";
import { listen } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { RadialMenuItem, ShowRadialMenuEvent } from "../types";

let items: Array<RadialMenuItem> = [];

export const radialMenuSize = 200;

listen<ShowRadialMenuEvent>('show-radial-menu', async (event) => {
  console.log("Received show-radial-menu event", event.payload.items);
  let radialMenu = await WebviewWindow.getByLabel("radial-menu");
  if (radialMenu === null) {
    console.log("No radial menu found");
    return;
  }

  items = event.payload.items; 

  let location = event.payload.location;
  await radialMenu.setPosition(new LogicalPosition(location[0] - radialMenuSize / 2, location[1] - radialMenuSize / 2));
  await radialMenu.show();
});

listen('hide-radial-menu', async () => {
  console.log("Received hide-radial-menu event");
  let radialMenu = await WebviewWindow.getByLabel("radial-menu");
  if (radialMenu === null) {
    console.log("No radial menu found");
    return;
  }

  await radialMenu.hide();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RadialMenu />
  </React.StrictMode>,
);
