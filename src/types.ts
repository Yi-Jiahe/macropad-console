export type ActiveWindow = {
  title: string;
  appName: string;
};

export type KeyPressAction = {
  key: string;
};

export type RadialMenuItem = {
  label: string;
  action: KeyPressAction;
}

export type ShowRadialMenuEvent = {
  location: [number, number];
  items: Array<RadialMenuItem>;
};
