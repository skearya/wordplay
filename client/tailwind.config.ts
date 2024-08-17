import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./src/**/*.{html,js,jsx,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#050705",
        "light-background": "#0C0D0A",
        foreground: "#FFFFFF",
        "lightest-green": "#62E297",
        "light-green": "#8BA698",
        green: "#26D16C",
        "dark-green": "#475D50",
        blue: "#345C8A",
      },
      fontFamily: {
        sans: ["Inter Variable", "sans-serif"],
        mono: ["JetBrains Mono Variable", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
