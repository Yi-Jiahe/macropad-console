export type ActiveWindow = {
  title: string;
  appName: string;
};

export type ApplicationConfig = {
  applicationProfiles: {[key: string]:  ApplicationProfile};
}

export type ApplicationProfile = {
  actions: Array<[Action, ApplicationAction]>
}

// Actions

export type ButtonIds = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11;
export type EncoderIds = 0;

export type Action = {
  buttonPress?: {
    button: ButtonIds;
  }
  encoderIncrement?: {
    id: EncoderIds;
  }
  encoderDecrement?: {
    id: EncoderIds;
  }
}

// Application Actions

export type ApplicationAction = {
  openRadialMenu?: {
    items: Array<RadialMenuItem>;
  }
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
  macroTap?: {
    actions: Array<ApplicationAction>;
  }
}

export type RadialMenuItem = {
  label: string;
  action: ApplicationAction;
}

// Events

export type OpenRadialMenu = {
  items: Array<RadialMenuItem>;
}


export type ShowRadialMenuEvent = {
  location: [number, number];
  items: Array<RadialMenuItem>;
};
