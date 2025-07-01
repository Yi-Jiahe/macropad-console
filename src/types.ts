export type ActiveWindow = {
  title: string;
  appName: string;
};

export type ApplicationConfig = {
  applicationProfiles: {[key: string]:  ApplicationProfile};
}

export type ApplicationProfile = {
  bindings: Array<[Action, Command]>
}

// Actions

export type ButtonIds = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11;
export type EncoderIds = 0;

export type Action = {
  buttonPress?: {
    id: ButtonIds;
  }
  encoderIncrement?: {
    id: EncoderIds;
  }
  encoderDecrement?: {
    id: EncoderIds;
  }
}

export type Command = {
  displayName: string;
  radialMenuItems?: Array<RadialMenuItem>;
  operations?: Array<Operation>;
}

export type Operation = {
  keyPress?: {
    key: string;
  };
  keyTap?: {
    key: string;
  };
  keyRelease?: {
    key: string;
  };
  delay?: {
    ms: number;
  };
  repeat?: {
    times: number;
    operations: Array<Operation>;
  };
}

export type RadialMenuItem = {
  label: string;
  command: Command;
}

// Events

export type ShowRadialMenuEvent = {
  location: [number, number];
  items: Array<RadialMenuItem>;
};
