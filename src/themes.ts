export interface Theme {
  name: string;
  icon: string;
  colors: {
    bgPrimary: string;
    bgSecondary: string;
    bgTertiary: string;
    textPrimary: string;
    textSecondary: string;
    accent1: string; // Primary accent
    accent2: string; // Secondary accent
    accent3: string; // Tertiary accent
    success: string;
    warning: string;
    error: string;
  };
}

export const themes: Record<string, Theme> = {
  cyberpunk: {
    name: "Cyberpunk",
    icon: "🌆",
    colors: {
      bgPrimary: "#0f0f23",
      bgSecondary: "#1a1a2e",
      bgTertiary: "#25253a",
      textPrimary: "#e4e4e7",
      textSecondary: "#a1a1aa",
      accent1: "#8b5cf6", // Purple
      accent2: "#22d3ee", // Cyan
      accent3: "#f472b6", // Pink
      success: "#22c55e",
      warning: "#eab308",
      error: "#ef4444",
    },
  },
  matrix: {
    name: "Matrix",
    icon: "💊",
    colors: {
      bgPrimary: "#0d0d0d",
      bgSecondary: "#141414",
      bgTertiary: "#1a1a1a",
      textPrimary: "#00ff00",
      textSecondary: "#008800",
      accent1: "#00ff00", // Neon green
      accent2: "#33ff33", // Light green
      accent3: "#00cc00", // Dark green
      success: "#00ff00",
      warning: "#ffff00",
      error: "#ff0000",
    },
  },
  dracula: {
    name: "Dracula",
    icon: "🧛",
    colors: {
      bgPrimary: "#282a36",
      bgSecondary: "#343746",
      bgTertiary: "#44475a",
      textPrimary: "#f8f8f2",
      textSecondary: "#6272a4",
      accent1: "#bd93f9", // Purple
      accent2: "#8be9fd", // Cyan
      accent3: "#ff79c6", // Pink
      success: "#50fa7b",
      warning: "#f1fa8c",
      error: "#ff5555",
    },
  },
  nord: {
    name: "Nord",
    icon: "❄️",
    colors: {
      bgPrimary: "#2e3440",
      bgSecondary: "#3b4252",
      bgTertiary: "#434c5e",
      textPrimary: "#eceff4",
      textSecondary: "#d8dee9",
      accent1: "#88c0d0", // Frost blue
      accent2: "#81a1c1", // Blue
      accent3: "#5e81ac", // Dark blue
      success: "#a3be8c",
      warning: "#ebcb8b",
      error: "#bf616a",
    },
  },
  synthwave: {
    name: "Synthwave",
    icon: "🌅",
    colors: {
      bgPrimary: "#1a1025",
      bgSecondary: "#241734",
      bgTertiary: "#2d1f42",
      textPrimary: "#ff71ce",
      textSecondary: "#b967ff",
      accent1: "#ff71ce", // Hot pink
      accent2: "#01cdfe", // Electric blue
      accent3: "#b967ff", // Purple
      success: "#05ffa1",
      warning: "#fffb96",
      error: "#ff6b6b",
    },
  },
  monokai: {
    name: "Monokai",
    icon: "🌙",
    colors: {
      bgPrimary: "#272822",
      bgSecondary: "#3e3d32",
      bgTertiary: "#49483e",
      textPrimary: "#f8f8f2",
      textSecondary: "#75715e",
      accent1: "#f92672", // Pink
      accent2: "#66d9ef", // Blue
      accent3: "#a6e22e", // Green
      success: "#a6e22e",
      warning: "#e6db74",
      error: "#f92672",
    },
  },
  catppuccin: {
    name: "Catppuccin",
    icon: "🐱",
    colors: {
      bgPrimary: "#1e1e2e",
      bgSecondary: "#302d41",
      bgTertiary: "#45425a",
      textPrimary: "#cdd6f4",
      textSecondary: "#a6adc8",
      accent1: "#cba6f7", // Mauve
      accent2: "#89dceb", // Sky
      accent3: "#f5c2e7", // Pink
      success: "#a6e3a1",
      warning: "#f9e2af",
      error: "#f38ba8",
    },
  },
  gruvbox: {
    name: "Gruvbox",
    icon: "🍂",
    colors: {
      bgPrimary: "#1d2021",
      bgSecondary: "#282828",
      bgTertiary: "#32302f",
      textPrimary: "#ebdbb2",
      textSecondary: "#a89984",
      accent1: "#fe8019", // Orange
      accent2: "#83a598", // Aqua
      accent3: "#b8bb26", // Yellow-green
      success: "#b8bb26",
      warning: "#fabd2f",
      error: "#fb4934",
    },
  },
  // ═══════════════════════════════════════════════════════════
  // VIBRANT & DARK THEMES
  // ═══════════════════════════════════════════════════════════
  neon: {
    name: "Neon",
    icon: "💜",
    colors: {
      bgPrimary: "#000000",
      bgSecondary: "#0a0a0a",
      bgTertiary: "#111111",
      textPrimary: "#ffffff",
      textSecondary: "#888888",
      accent1: "#ff00ff", // Magenta vif
      accent2: "#00ffff", // Cyan électrique
      accent3: "#ff00aa", // Rose néon
      success: "#00ff88",
      warning: "#ffff00",
      error: "#ff0044",
    },
  },
  bloodmoon: {
    name: "Blood Moon",
    icon: "🩸",
    colors: {
      bgPrimary: "#0a0000",
      bgSecondary: "#120000",
      bgTertiary: "#1a0505",
      textPrimary: "#ffcccc",
      textSecondary: "#aa6666",
      accent1: "#ff0033", // Rouge sang
      accent2: "#ff4466", // Rouge clair
      accent3: "#cc0022", // Rouge foncé
      success: "#00ff66",
      warning: "#ff8800",
      error: "#ff0000",
    },
  },
  hacker: {
    name: "Hacker",
    icon: "👾",
    colors: {
      bgPrimary: "#000000",
      bgSecondary: "#001100",
      bgTertiary: "#002200",
      textPrimary: "#00ff00",
      textSecondary: "#00aa00",
      accent1: "#00ff00", // Vert terminal
      accent2: "#00ffaa", // Cyan-vert
      accent3: "#88ff00", // Vert-jaune
      success: "#00ff00",
      warning: "#ffff00",
      error: "#ff0000",
    },
  },
  vaporwave: {
    name: "Vaporwave",
    icon: "🌴",
    colors: {
      bgPrimary: "#0f0020",
      bgSecondary: "#150030",
      bgTertiary: "#1a0040",
      textPrimary: "#ff71ce",
      textSecondary: "#b967ff",
      accent1: "#ff6ac1", // Pink saturé
      accent2: "#00f5d4", // Turquoise vif
      accent3: "#7b2cbf", // Violet profond
      success: "#00f5d4",
      warning: "#fee440",
      error: "#ff006e",
    },
  },
  tokyonight: {
    name: "Tokyo Night",
    icon: "🗼",
    colors: {
      bgPrimary: "#0a0e14",
      bgSecondary: "#0d1117",
      bgTertiary: "#161b22",
      textPrimary: "#c9d1d9",
      textSecondary: "#7d8590",
      accent1: "#58a6ff", // Bleu électrique
      accent2: "#f778ba", // Rose Tokyo
      accent3: "#7ee787", // Vert néon
      success: "#7ee787",
      warning: "#d29922",
      error: "#f85149",
    },
  },
  abyss: {
    name: "Abyss",
    icon: "🕳️",
    colors: {
      bgPrimary: "#000005",
      bgSecondary: "#00000f",
      bgTertiary: "#000019",
      textPrimary: "#e0e0ff",
      textSecondary: "#6666aa",
      accent1: "#0066ff", // Bleu abysse
      accent2: "#00ccff", // Cyan profond
      accent3: "#6600ff", // Violet électrique
      success: "#00ff99",
      warning: "#ffcc00",
      error: "#ff3366",
    },
  },
  inferno: {
    name: "Inferno",
    icon: "🔥",
    colors: {
      bgPrimary: "#0a0000",
      bgSecondary: "#100500",
      bgTertiary: "#180a00",
      textPrimary: "#ffddcc",
      textSecondary: "#aa7755",
      accent1: "#ff6600", // Orange feu
      accent2: "#ffcc00", // Jaune flamme
      accent3: "#ff3300", // Rouge braise
      success: "#88ff00",
      warning: "#ffcc00",
      error: "#ff0000",
    },
  },
  midnight: {
    name: "Midnight",
    icon: "🌑",
    colors: {
      bgPrimary: "#000011",
      bgSecondary: "#000822",
      bgTertiary: "#001133",
      textPrimary: "#e8e8ff",
      textSecondary: "#8888bb",
      accent1: "#4488ff", // Bleu nuit
      accent2: "#88ccff", // Bleu ciel
      accent3: "#aa66ff", // Violet
      success: "#44ff88",
      warning: "#ffaa44",
      error: "#ff4466",
    },
  },
  toxic: {
    name: "Toxic",
    icon: "☢️",
    colors: {
      bgPrimary: "#000800",
      bgSecondary: "#001000",
      bgTertiary: "#001800",
      textPrimary: "#ccffcc",
      textSecondary: "#66aa66",
      accent1: "#44ff00", // Vert toxique
      accent2: "#aaff00", // Jaune-vert
      accent3: "#00ff44", // Vert vif
      success: "#00ff00",
      warning: "#ffff00",
      error: "#ff4400",
    },
  },
  ultraviolet: {
    name: "Ultraviolet",
    icon: "🔮",
    colors: {
      bgPrimary: "#05000a",
      bgSecondary: "#0a0014",
      bgTertiary: "#10001e",
      textPrimary: "#e8ccff",
      textSecondary: "#9966cc",
      accent1: "#9900ff", // Violet UV
      accent2: "#ff00ff", // Magenta
      accent3: "#6600cc", // Violet profond
      success: "#00ff99",
      warning: "#ffcc00",
      error: "#ff0066",
    },
  },
  redshift: {
    name: "Redshift",
    icon: "🌋",
    colors: {
      bgPrimary: "#080000",
      bgSecondary: "#100404",
      bgTertiary: "#180808",
      textPrimary: "#ffcccc",
      textSecondary: "#cc8888",
      accent1: "#ff2200", // Rouge intense
      accent2: "#ff6644", // Orange-rouge
      accent3: "#cc0000", // Rouge sombre
      success: "#44ff44",
      warning: "#ffaa00",
      error: "#ff0000",
    },
  },
  electric: {
    name: "Electric",
    icon: "⚡",
    colors: {
      bgPrimary: "#000008",
      bgSecondary: "#000410",
      bgTertiary: "#000818",
      textPrimary: "#ffffff",
      textSecondary: "#88aacc",
      accent1: "#00aaff", // Bleu électrique
      accent2: "#00ffff", // Cyan
      accent3: "#0066ff", // Bleu vif
      success: "#00ff88",
      warning: "#ffdd00",
      error: "#ff4444",
    },
  },
};

export const themeKeys = Object.keys(themes);

export const applyTheme = (themeKey: string) => {
  const theme = themes[themeKey] || themes.cyberpunk;
  const root = document.documentElement;

  root.style.setProperty("--bg-primary", theme.colors.bgPrimary);
  root.style.setProperty("--bg-secondary", theme.colors.bgSecondary);
  root.style.setProperty("--bg-tertiary", theme.colors.bgTertiary);
  root.style.setProperty("--text-primary", theme.colors.textPrimary);
  root.style.setProperty("--text-secondary", theme.colors.textSecondary);
  root.style.setProperty("--accent-1", theme.colors.accent1);
  root.style.setProperty("--accent-2", theme.colors.accent2);
  root.style.setProperty("--accent-3", theme.colors.accent3);
  root.style.setProperty("--success", theme.colors.success);
  root.style.setProperty("--warning", theme.colors.warning);
  root.style.setProperty("--error", theme.colors.error);
};

export const getStoredTheme = (): string => {
  return localStorage.getItem("claude-dashboard-theme") || "cyberpunk";
};

export const storeTheme = (themeKey: string) => {
  localStorage.setItem("claude-dashboard-theme", themeKey);
};
