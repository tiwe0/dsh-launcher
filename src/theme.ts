import { createTheme } from "@mui/material/styles";

export const launcherTheme = createTheme({
  cssVariables: { nativeColor: true },
  palette: {
    mode: "dark",
    primary: {
      main: "#6799fe",
      dark: "#4d6bfe",
      light: "#93b7ff",
      contrastText: "#07101e",
    },
    success: {
      main: "oklch(0.56 0.13 155)",
      contrastText: "oklch(1 0 0)",
    },
    warning: {
      main: "oklch(0.68 0.14 72)",
      contrastText: "oklch(0.18 0.01 72)",
    },
    error: {
      main: "oklch(0.56 0.18 27)",
      contrastText: "oklch(1 0 0)",
    },
    background: {
      default: "#0a0a0a",
      paper: "#17191e",
    },
    text: {
      primary: "#ffffff",
      secondary: "rgba(255, 255, 255, 0.62)",
    },
    divider: "rgba(255, 255, 255, 0.1)",
  },
  shape: { borderRadius: 10 },
  typography: {
    fontFamily:
      'Inter, ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    h1: { fontSize: "1.75rem", lineHeight: 1.2, fontWeight: 720, letterSpacing: "-0.025em" },
    h2: { fontSize: "1.08rem", lineHeight: 1.35, fontWeight: 700, letterSpacing: "-0.012em" },
    subtitle1: { fontSize: "0.96rem", fontWeight: 650 },
    body1: { fontSize: "0.92rem", lineHeight: 1.55 },
    body2: { fontSize: "0.82rem", lineHeight: 1.5 },
    button: { fontSize: "0.84rem", fontWeight: 680, textTransform: "none" },
  },
  components: {
    MuiButton: {
      defaultProps: { disableElevation: true },
      styleOverrides: {
        root: { minHeight: 38, borderRadius: 9 },
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        root: {
          background: "rgba(255, 255, 255, 0.055)",
          "& .MuiOutlinedInput-notchedOutline": { borderColor: "rgba(255, 255, 255, 0.16)" },
          "&:hover .MuiOutlinedInput-notchedOutline": { borderColor: "rgba(255, 255, 255, 0.28)" },
          "&.Mui-focused .MuiOutlinedInput-notchedOutline": { borderColor: "#6799fe" },
        },
      },
    },
    MuiDialog: {
      styleOverrides: {
        paper: { backgroundImage: "none", border: "1px solid rgba(255, 255, 255, 0.1)" },
      },
    },
    MuiTextField: {
      defaultProps: { size: "small" },
    },
    MuiInputBase: {
      styleOverrides: { root: { fontSize: "0.875rem" } },
    },
    MuiTooltip: {
      defaultProps: { arrow: true },
    },
  },
});
