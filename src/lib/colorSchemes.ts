export enum AccentColorName {
  Blue = 'blue',
  Green = 'green',
  Orange = 'orange',
  Pink = 'pink',
  Purple = 'purple',
  Red = 'red',
  Yellow = 'yellow',
}

export enum ColorSchemeName {
  CatppuccinFrappe = 'catppuccin-frappe',
  CatppuccinLatte = 'catppuccin-latte',
  CatppuccinMacchiato = 'catppuccin-macchiato',
  CatppuccinMocha = 'catppuccin-mocha',
  DefaultDark = 'default-dark',
  DefaultLight = 'default-light',
  GruvboxDark = 'gruvbox-dark',
  GruvboxLight = 'gruvbox-light',
}

export interface AccentColor {
  foreground: string
  hex:        string
  name:       AccentColorName
}

export interface ColorScheme {
  accentColors: AccentColor[]
  colors: {
    accent:                string
    accentForeground:      string
    background:            string
    backgroundDark:        string
    border:                string
    card:                  string
    cardForeground:        string
    destructive:           string
    destructiveForeground: string
    foreground:            string
    input:                 string
    muted:                 string
    mutedForeground:       string
    popover:               string
    popoverForeground:     string
    primary:               string
    primaryForeground:     string
    ring:                  string
    secondary:             string
    secondaryForeground:   string
    sidebar:               string
    success:               string
  }
  name: ColorSchemeName
}

export const COLOR_SCHEMES: ColorScheme[] = [
  {
    accentColors: [
      { foreground: '#fafafa', hex: '#ef4444', name: AccentColorName.Red },
      { foreground: '#18181b', hex: '#f97316', name: AccentColorName.Orange },
      { foreground: '#18181b', hex: '#f59e0b', name: AccentColorName.Yellow },
      { foreground: '#18181b', hex: '#10b981', name: AccentColorName.Green },
      { foreground: '#fafafa', hex: '#3b82f6', name: AccentColorName.Blue },
      { foreground: '#fafafa', hex: '#8b5cf6', name: AccentColorName.Purple },
      { foreground: '#fafafa', hex: '#ec4899', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#64748b',
      accentForeground:      '#fafafa',
      background:            '#ffffff',
      backgroundDark:        '#f8f8f8',
      border:                '#e4e4e7',
      card:                  '#ffffff',
      cardForeground:        '#09090b',
      destructive:           '#e7000b',
      destructiveForeground: '#fafafa',
      foreground:            '#09090b',
      input:                 '#e4e4e7',
      muted:                 '#f4f4f5',
      mutedForeground:       '#71717b',
      popover:               '#ffffff',
      popoverForeground:     '#09090b',
      primary:               '#18181b',
      primaryForeground:     '#fafafa',
      ring:                  '#9f9fa9',
      secondary:             '#f4f4f5',
      secondaryForeground:   '#18181b',
      sidebar:               '#cdcdd4',
      success:               '#10b981',
    },

    name: ColorSchemeName.DefaultLight,
  },
  {
    accentColors: [
      { foreground: '#18181b', hex: '#ef4444', name: AccentColorName.Red },
      { foreground: '#18181b', hex: '#f97316', name: AccentColorName.Orange },
      { foreground: '#18181b', hex: '#f59e0b', name: AccentColorName.Yellow },
      { foreground: '#18181b', hex: '#10b981', name: AccentColorName.Green },
      { foreground: '#18181b', hex: '#3b82f6', name: AccentColorName.Blue },
      { foreground: '#18181b', hex: '#8b5cf6', name: AccentColorName.Purple },
      { foreground: '#18181b', hex: '#ec4899', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#64748b',
      accentForeground:      '#fafafa',
      background:            '#09090b',
      backgroundDark:        '#030304',
      border:                '#27272a',
      card:                  '#09090b',
      cardForeground:        '#fafafa',
      destructive:           '#ef4444',
      destructiveForeground: '#09090b',
      foreground:            '#fafafa',
      input:                 '#27272a',
      muted:                 '#27272a',
      mutedForeground:       '#9f9fa9',
      popover:               '#09090b',
      popoverForeground:     '#fafafa',
      primary:               '#fafafa',
      primaryForeground:     '#18181b',
      ring:                  '#52525c',
      secondary:             '#27272a',
      secondaryForeground:   '#fafafa',
      sidebar:               '#000000',
      success:               '#10b981',
    },

    name: ColorSchemeName.DefaultDark,
  },
  {
    accentColors: [
      { foreground: '#eff1f5', hex: '#d20f39', name: AccentColorName.Red },
      { foreground: '#4c4f69', hex: '#fe640b', name: AccentColorName.Orange },
      { foreground: '#4c4f69', hex: '#df8e1d', name: AccentColorName.Yellow },
      { foreground: '#4c4f69', hex: '#40a02b', name: AccentColorName.Green },
      { foreground: '#eff1f5', hex: '#1e66f5', name: AccentColorName.Blue },
      { foreground: '#eff1f5', hex: '#8839ef', name: AccentColorName.Purple },
      { foreground: '#4c4f69', hex: '#ea76cb', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#dc8a78',
      accentForeground:      '#eff1f5',
      background:            '#eff1f5',
      backgroundDark:        '#e6e9ef',
      border:                '#9ca0b0',
      card:                  '#eff1f5',
      cardForeground:        '#4c4f69',
      destructive:           '#d20f39',
      destructiveForeground: '#eff1f5',
      foreground:            '#4c4f69',
      input:                 '#9ca0b0',
      muted:                 '#bcc0cc',
      mutedForeground:       '#6c6f85',
      popover:               '#eff1f5',
      popoverForeground:     '#4c4f69',
      primary:               '#1e66f5',
      primaryForeground:     '#eff1f5',
      ring:                  '#1e66f5',
      secondary:             '#acb0be',
      secondaryForeground:   '#4c4f69',
      sidebar:               '#d1d5db',
      success:               '#40a02b',
    },

    name: ColorSchemeName.CatppuccinLatte,
  },
  {
    accentColors: [
      { foreground: '#303446', hex: '#e78284', name: AccentColorName.Red },
      { foreground: '#303446', hex: '#f2a66f', name: AccentColorName.Orange },
      { foreground: '#303446', hex: '#e5c890', name: AccentColorName.Yellow },
      { foreground: '#303446', hex: '#a6d189', name: AccentColorName.Green },
      { foreground: '#303446', hex: '#8caaee', name: AccentColorName.Blue },
      { foreground: '#303446', hex: '#ca9ee6', name: AccentColorName.Purple },
      { foreground: '#303446', hex: '#f4b8e4', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#f2d5cf',
      accentForeground:      '#303446',
      background:            '#303446',
      backgroundDark:        '#1a1a2e',
      border:                '#626880',
      card:                  '#303446',
      cardForeground:        '#c6d0f5',
      destructive:           '#e78284',
      destructiveForeground: '#303446',
      foreground:            '#c6d0f5',
      input:                 '#626880',
      muted:                 '#737994',
      mutedForeground:       '#949cbb',
      popover:               '#303446',
      popoverForeground:     '#c6d0f5',
      primary:               '#8caaee',
      primaryForeground:     '#303446',
      ring:                  '#8caaee',
      secondary:             '#626880',
      secondaryForeground:   '#c6d0f5',
      sidebar:               '#292c3c',
      success:               '#a6d189',
    },

    name: ColorSchemeName.CatppuccinFrappe,
  },
  {
    accentColors: [
      { foreground: '#24273a', hex: '#ed8796', name: AccentColorName.Red },
      { foreground: '#24273a', hex: '#f5a97f', name: AccentColorName.Orange },
      { foreground: '#24273a', hex: '#eed49f', name: AccentColorName.Yellow },
      { foreground: '#24273a', hex: '#a6da95', name: AccentColorName.Green },
      { foreground: '#24273a', hex: '#8aadf4', name: AccentColorName.Blue },
      { foreground: '#24273a', hex: '#c6a0f6', name: AccentColorName.Purple },
      { foreground: '#24273a', hex: '#f5bde6', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#f4dbd6',
      accentForeground:      '#24273a',
      background:            '#24273a',
      backgroundDark:        '#1a1a2e',
      border:                '#5b6078',
      card:                  '#24273a',
      cardForeground:        '#cad3f5',
      destructive:           '#ed8796',
      destructiveForeground: '#24273a',
      foreground:            '#cad3f5',
      input:                 '#5b6078',
      muted:                 '#6e738d',
      mutedForeground:       '#a5adcb',
      popover:               '#24273a',
      popoverForeground:     '#cad3f5',
      primary:               '#8aadf4',
      primaryForeground:     '#24273a',
      ring:                  '#8aadf4',
      secondary:             '#5b6078',
      secondaryForeground:   '#cad3f5',
      sidebar:               '#1e1e2e',
      success:               '#a6da95',
    },

    name: ColorSchemeName.CatppuccinMacchiato,
  },
  {
    accentColors: [
      { foreground: '#1e1e2e', hex: '#f38ba8', name: AccentColorName.Red },
      { foreground: '#1e1e2e', hex: '#fab387', name: AccentColorName.Orange },
      { foreground: '#1e1e2e', hex: '#f9e2af', name: AccentColorName.Yellow },
      { foreground: '#1e1e2e', hex: '#a6e3a1', name: AccentColorName.Green },
      { foreground: '#1e1e2e', hex: '#89b4fa', name: AccentColorName.Blue },
      { foreground: '#1e1e2e', hex: '#cba6f7', name: AccentColorName.Purple },
      { foreground: '#1e1e2e', hex: '#f5c2e7', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#f5e0dc',
      accentForeground:      '#1e1e2e',
      background:            '#1e1e2e',
      backgroundDark:        '#11111b',
      border:                '#585b70',
      card:                  '#1e1e2e',
      cardForeground:        '#cdd6f4',
      destructive:           '#f38ba8',
      destructiveForeground: '#1e1e2e',
      foreground:            '#cdd6f4',
      input:                 '#585b70',
      muted:                 '#6c7086',
      mutedForeground:       '#a6adc8',
      popover:               '#1e1e2e',
      popoverForeground:     '#cdd6f4',
      primary:               '#89b4fa',
      primaryForeground:     '#1e1e2e',
      ring:                  '#89b4fa',
      secondary:             '#585b70',
      secondaryForeground:   '#cdd6f4',
      sidebar:               '#181825',
      success:               '#a6e3a1',
    },

    name: ColorSchemeName.CatppuccinMocha,
  },
  {
    accentColors: [
      { foreground: '#fbf1c7', hex: '#cc241d', name: AccentColorName.Red },
      { foreground: '#fbf1c7', hex: '#d65d0e', name: AccentColorName.Orange },
      { foreground: '#3c3836', hex: '#d79921', name: AccentColorName.Yellow },
      { foreground: '#fbf1c7', hex: '#98971a', name: AccentColorName.Green },
      { foreground: '#fbf1c7', hex: '#458588', name: AccentColorName.Blue },
      { foreground: '#fbf1c7', hex: '#b16286', name: AccentColorName.Purple },
      { foreground: '#3c3836', hex: '#d3869b', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#d79921',
      accentForeground:      '#fbf1c7',
      background:            '#fbf1c7',
      backgroundDark:        '#f2e6b6',
      border:                '#bdae93',
      card:                  '#fbf1c7',
      cardForeground:        '#3c3836',
      destructive:           '#cc241d',
      destructiveForeground: '#fbf1c7',
      foreground:            '#3c3836',
      input:                 '#bdae93',
      muted:                 '#d5c4a1',
      mutedForeground:       '#665c54',
      popover:               '#fbf1c7',
      popoverForeground:     '#3c3836',
      primary:               '#458588',
      primaryForeground:     '#fbf1c7',
      ring:                  '#458588',
      secondary:             '#ebdbb2',
      secondaryForeground:   '#3c3836',
      sidebar:               '#d5c4a1',
      success:               '#98971a',
    },

    name: ColorSchemeName.GruvboxLight,
  },
  {
    accentColors: [
      { foreground: '#282828', hex: '#fb4934', name: AccentColorName.Red },
      { foreground: '#282828', hex: '#fe8019', name: AccentColorName.Orange },
      { foreground: '#282828', hex: '#fabd2f', name: AccentColorName.Yellow },
      { foreground: '#282828', hex: '#b8bb26', name: AccentColorName.Green },
      { foreground: '#282828', hex: '#83a598', name: AccentColorName.Blue },
      { foreground: '#282828', hex: '#d3869b', name: AccentColorName.Purple },
      { foreground: '#282828', hex: '#d3869b', name: AccentColorName.Pink },
    ],
    colors: {
      accent:                '#fabd2f',
      accentForeground:      '#282828',
      background:            '#282828',
      backgroundDark:        '#1a1a1a',
      border:                '#504945',
      card:                  '#282828',
      cardForeground:        '#ebdbb2',
      destructive:           '#fb4934',
      destructiveForeground: '#282828',
      foreground:            '#ebdbb2',
      input:                 '#504945',
      muted:                 '#665c54',
      mutedForeground:       '#a89984',
      popover:               '#282828',
      popoverForeground:     '#ebdbb2',
      primary:               '#83a598',
      primaryForeground:     '#282828',
      ring:                  '#83a598',
      secondary:             '#504945',
      secondaryForeground:   '#ebdbb2',
      sidebar:               '#1d2021',
      success:               '#b8bb26',
    },

    name: ColorSchemeName.GruvboxDark,
  },
]
