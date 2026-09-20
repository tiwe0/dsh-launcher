import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Alert,
  Box,
  Button,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from "@mui/material";
import {
  AddRounded,
  ArrowOutwardRounded,
  CheckRounded,
  CloseRounded,
  DeleteOutlineRounded,
  DownloadRounded,
  FolderOpenRounded,
  GitHub,
  RemoveRounded,
  PowerSettingsNewRounded,
  RefreshRounded,
  RestartAltRounded,
  StopRounded,
  TranslateRounded,
} from "@mui/icons-material";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useTranslation } from "react-i18next";
import "./App.css";

const DEFAULT_NODE_VERSION = "24.21.0";
const NODE_VERSIONS = ["24.21.0", "26.9.0"];
const DEFAULT_DSH_VERSION = "0.1.6-alpha.2";

type RuntimeStatus = {
  ready: boolean;
  running: boolean;
  nodeVersion: string | null;
  dshVersion: string | null;
  endpoint: string | null;
  pid: number | null;
};

type DshVersion = {
  version: string;
  installed: boolean;
  isDefault: boolean;
  isRunning: boolean;
};

type RuntimeStateEvent = {
  status: "setting-up" | "ready" | "starting" | "running" | "stopping" | "stopped" | "error";
  pid?: number | null;
  endpoint?: string | null;
  message?: string;
};

const EMPTY_STATUS: RuntimeStatus = {
  ready: false,
  running: false,
  nodeVersion: null,
  dshVersion: null,
  endpoint: null,
  pid: null,
};

function normalizeStatus(value: Record<string, unknown>): RuntimeStatus {
  return {
    ready: Boolean(value.ready),
    running: Boolean(value.running),
    nodeVersion: (value.nodeVersion ?? value.node_version ?? null) as string | null,
    dshVersion: (value.dshVersion ?? value.dsh_version ?? null) as string | null,
    endpoint: (value.endpoint ?? null) as string | null,
    pid: (value.pid ?? null) as number | null,
  };
}

function normalizeVersions(values: Array<Record<string, unknown>>): DshVersion[] {
  return values.map((value) => ({
    version: String(value.version),
    installed: value.status !== "incomplete",
    isDefault: Boolean(value.isDefault ?? value.is_default),
    isRunning: Boolean(value.isRunning ?? value.is_running ?? value.active),
  }));
}

function errorText(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

function displayWorkspacePath(path: string) {
  const isDefaultUnixWorkspace = /^\/(?:Users|home)\/[^/]+\/\.dsh-launcher$/.test(path);
  const isDefaultWindowsWorkspace = /^[A-Za-z]:\\Users\\[^\\]+\\\.dsh-launcher$/.test(path);
  return isDefaultUnixWorkspace || isDefaultWindowsWorkspace ? "~/.dsh-launcher" : path;
}

export default function App() {
  const { t, i18n } = useTranslation();
  const [status, setStatus] = useState(EMPTY_STATUS);
  const [versions, setVersions] = useState<DshVersion[]>([]);
  const [remoteVersions, setRemoteVersions] = useState<string[]>([]);
  const [nodeVersion, setNodeVersion] = useState(DEFAULT_NODE_VERSION);
  const [dshVersion, setDshVersion] = useState(DEFAULT_DSH_VERSION);
  const [workspace, setWorkspace] = useState("");
  const [busy, setBusy] = useState<"setup" | "start" | "stop" | "restart" | "check" | "install" | "remove" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastMessageKey, setLastMessageKey] = useState("status.notCreated");
  const [versionDialog, setVersionDialog] = useState(false);
  const [newVersion, setNewVersion] = useState(DEFAULT_DSH_VERSION);
  const [languageTurns, setLanguageTurns] = useState(0);
  const [languageChanging, setLanguageChanging] = useState(false);
  const restartPending = useRef(false);
  const languageSwapTimer = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [rawStatus, rawVersions] = await Promise.all([
        invoke<Record<string, unknown>>("runtime_status"),
        invoke<Array<Record<string, unknown>>>("list_dsh_versions"),
      ]);
      const nextStatus = normalizeStatus(rawStatus);
      const nextVersions = normalizeVersions(rawVersions);
      setStatus(nextStatus);
      setVersions(nextVersions);
      setNodeVersion(nextStatus.nodeVersion ?? DEFAULT_NODE_VERSION);
      const preferred = nextStatus.dshVersion ?? nextVersions.find((item) => item.isDefault)?.version ?? nextVersions[0]?.version;
      if (preferred) setDshVersion(preferred);
      setLastMessageKey(nextStatus.running ? "status.running" : nextStatus.ready ? "status.ready" : "status.notCreated");
    } catch {
      // The browser-only Vite preview has no native backend.
    }
  }, []);

  useEffect(() => {
    void refresh();
    if (!("__TAURI_INTERNALS__" in window)) return;

    void invoke<string>("default_workspace")
      .then((directory) => setWorkspace((current) => current || directory))
      .catch((workspaceError) => setError(errorText(workspaceError)));

    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<RuntimeStateEvent>("runtime-state", (event) => {
      if (disposed) return;
      const payload = event.payload;
      if (payload.status === "running") {
        restartPending.current = false;
        setStatus((current) => ({ ...current, running: true, pid: payload.pid ?? current.pid, endpoint: payload.endpoint ?? current.endpoint }));
        setBusy(null);
        setLastMessageKey("status.started");
        if (payload.endpoint) void openUrl(payload.endpoint);
      } else if (payload.status === "stopped") {
        setStatus((current) => ({ ...current, running: false, pid: null, endpoint: null }));
        if (restartPending.current) {
          setBusy("restart");
          setLastMessageKey("status.restarting");
        } else {
          setBusy(null);
          setLastMessageKey("status.stopped");
        }
      } else if (payload.status === "error") {
        setBusy(null);
        setError(payload.message ?? t("errors.runtime"));
      } else if (payload.status === "starting") {
        setLastMessageKey("status.waiting");
      } else if (payload.status === "stopping") {
        setLastMessageKey("status.stopping");
      }
    })
      .then((handler) => {
        if (disposed) handler();
        else unlisten = handler;
      })
      .catch(() => {
        // The Vite-only visual preview does not expose Tauri events.
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refresh, t]);

  useEffect(() => () => {
    if (languageSwapTimer.current !== null) window.clearTimeout(languageSwapTimer.current);
  }, []);

  const selectedInstalled = useMemo(() => versions.some((item) => item.version === dshVersion && item.installed), [dshVersion, versions]);

  async function chooseWorkspace() {
    const selected = await open({ directory: true, multiple: false, title: t("launcher.chooseWorkspaceDialog") });
    if (typeof selected === "string") setWorkspace(selected);
  }

  async function handleToggle() {
    setError(null);
    if (status.running || busy === "start") {
      setBusy("stop");
      try {
        await invoke("stop_dsh");
      } catch (actionError) {
        setBusy(null);
        setError(errorText(actionError));
      }
      return;
    }
    if (!workspace) {
      setError(t("errors.selectWorkspace"));
      return;
    }

    try {
      let launchVersion = dshVersion;
      if (!status.ready || !selectedInstalled || status.nodeVersion !== nodeVersion) {
        setBusy("setup");
        setLastMessageKey("status.preparingRuntime");
        const result = await invoke<Record<string, unknown>>("setup_runtime", { nodeVersion });
        const prepared = normalizeStatus(result);
        launchVersion = prepared.dshVersion ?? DEFAULT_DSH_VERSION;
        setStatus(prepared);
        setDshVersion(launchVersion);
        await refresh();
      }

      setBusy("start");
      setLastMessageKey("status.starting");
      await invoke("start_dsh", {
        request: {
          version: launchVersion,
          nodeVersion,
          profile: "web",
          workspace,
          args: ["--no-open", "--port", "0"],
        },
      });
      restartPending.current = false;
    } catch (actionError) {
      setBusy(null);
      setError(errorText(actionError));
    }
  }

  function waitForStoppedAfterStop() {
    return new Promise<void>((resolve, reject) => {
      let unlisten: (() => void) | undefined;
      void listen<RuntimeStateEvent>("runtime-state", (event) => {
        if (event.payload.status === "stopped") {
          unlisten?.();
          resolve();
        } else if (event.payload.status === "error") {
          unlisten?.();
          reject(new Error(event.payload.message ?? t("errors.runtime")));
        }
      })
        .then((handler) => {
          unlisten = handler;
          return invoke("stop_dsh");
        })
        .catch((stopError) => {
          unlisten?.();
          reject(stopError);
        });
    });
  }

  async function handleRestart() {
    setError(null);
    restartPending.current = true;
    setBusy("restart");
    setLastMessageKey("status.restarting");
    try {
      await waitForStoppedAfterStop();
      await invoke("start_dsh", {
        request: {
          version: dshVersion,
          nodeVersion,
          profile: "web",
          workspace,
          args: ["--no-open", "--port", "0"],
        },
      });
    } catch (actionError) {
      restartPending.current = false;
      setBusy(null);
      setError(errorText(actionError));
    }
  }

  async function installVersion() {
    setError(null);
    setBusy("install");
    try {
      if (!status.ready) await invoke("setup_runtime", { nodeVersion });
      await invoke("install_dsh_version", { version: newVersion.trim() });
      setDshVersion(newVersion.trim());
      setVersionDialog(false);
      await refresh();
    } catch (actionError) {
      setError(errorText(actionError));
    } finally {
      setBusy(null);
    }
  }

  async function checkRemoteVersions() {
    setError(null);
    setBusy("check");
    try {
      if (!status.ready) {
        setLastMessageKey("status.preparingVersionCheck");
        await invoke("setup_runtime", { nodeVersion });
        await refresh();
      }
      const available = await invoke<string[]>("list_remote_dsh_versions");
      setRemoteVersions(available);
      const candidate = available.find((version) => !versions.some((item) => item.version === version && item.installed));
      if (candidate) setNewVersion(candidate);
    } catch (actionError) {
      setError(errorText(actionError));
    } finally {
      setBusy(null);
    }
  }

  async function removeVersion(version: string) {
    setBusy("remove");
    setError(null);
    try {
      await invoke("remove_dsh_version", { version });
      await refresh();
    } catch (actionError) {
      setError(errorText(actionError));
    } finally {
      setBusy(null);
    }
  }

  async function setDefaultVersion(version: string) {
    setBusy("install");
    setError(null);
    try {
      await invoke("set_default_dsh_version", { version });
      setDshVersion(version);
      await refresh();
    } catch (actionError) {
      setError(errorText(actionError));
    } finally {
      setBusy(null);
    }
  }

  const launching = busy === "setup" || busy === "start";
  const stopping = busy === "stop";
  const restarting = busy === "restart";
  const showRuntimeActions = status.running || stopping || restarting;
  const activeLanguage = (i18n.resolvedLanguage ?? i18n.language).startsWith("zh") ? "zh" : "en";
  const languageButtonLabel = activeLanguage === "zh" ? t("language.toggleToEnglish") : t("language.toggleToChinese");
  const harnessUrl = activeLanguage === "zh" ? "https://www.deepseek.com/harness/" : "https://www.deepseek.com/harness/en";

  function toggleLanguage() {
    if (languageChanging) return;
    const nextLanguage = activeLanguage === "zh" ? "en" : "zh";
    setLanguageChanging(true);
    setLanguageTurns((turns) => turns + 1);
    languageSwapTimer.current = window.setTimeout(() => {
      void i18n.changeLanguage(nextLanguage);
      window.requestAnimationFrame(() => setLanguageChanging(false));
      languageSwapTimer.current = null;
    }, 140);
  }

  async function minimizeWindow() {
    if ("__TAURI_INTERNALS__" in window) await getCurrentWindow().minimize();
  }

  async function closeWindow() {
    if ("__TAURI_INTERNALS__" in window) await getCurrentWindow().close();
  }

  return (
    <Box className={`page ${languageChanging ? "is-language-changing" : ""}`}>
      <header className="window-chrome" data-tauri-drag-region aria-label={t("window.controls")}>
        <button className="brand-lockup brand-link" type="button" aria-label={t("brand.openHarness")} onClick={() => void openUrl(harnessUrl)}>
          <img className="brand-mark" src="/deepseek-mark.svg" alt="" />
          <span className="brand-name">deepseek</span>
          <span className="brand-badge">{t("brand.badge")}</span>
        </button>
        <div className="chrome-tools">
          <Tooltip title={t("links.openGitHub")}>
            <IconButton className="window-action" size="small" aria-label={t("links.openGitHub")} onClick={() => void openUrl("https://github.com/tiwe0/dsh-launcher")}>
              <GitHub fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title={languageButtonLabel}>
            <IconButton className="window-action language-button" size="small" aria-label={languageButtonLabel} onClick={toggleLanguage} disabled={languageChanging}>
              <span className="language-icon" style={{ transform: `rotate(${languageTurns * 360}deg)` }}>
                <TranslateRounded fontSize="small" />
              </span>
            </IconButton>
          </Tooltip>
          <div className="window-actions">
            <IconButton className="window-action" size="small" aria-label={t("window.minimize")} onClick={() => void minimizeWindow()}>
              <RemoveRounded fontSize="small" />
            </IconButton>
            <IconButton className="window-action close-action" size="small" aria-label={t("window.close")} onClick={() => void closeWindow()}>
              <CloseRounded fontSize="small" />
            </IconButton>
          </div>
        </div>
      </header>

      <main className="launcher-frame">
        <section className="selectors" aria-label={t("launcher.config")}>
          <Box>
            <Typography component="h1" variant="h1">{t("launcher.title")}</Typography>
            <Typography color="text.secondary" mt={0.8}>{t("launcher.subtitle")}</Typography>
          </Box>

          {error && <Alert severity="error" onClose={() => setError(null)}>{error}</Alert>}

          <FormControl fullWidth size="small">
            <InputLabel id="node-version-label">{t("launcher.nodeVersion")}</InputLabel>
            <Select labelId="node-version-label" label={t("launcher.nodeVersion")} value={nodeVersion} onChange={(event) => setNodeVersion(event.target.value)} disabled={Boolean(busy) || status.running}>
              {NODE_VERSIONS.map((version) => <MenuItem key={version} value={version}>{version} · {version === DEFAULT_NODE_VERSION ? t("launcher.recommended") : t("launcher.onDemand")}</MenuItem>)}
            </Select>
          </FormControl>

          <Stack direction="row" spacing={1} alignItems="center">
            <FormControl fullWidth size="small">
              <InputLabel id="dsh-version-label">{t("launcher.dshVersion")}</InputLabel>
              <Select labelId="dsh-version-label" label={t("launcher.dshVersion")} value={dshVersion} onChange={(event) => setDshVersion(event.target.value)} disabled={Boolean(busy) || status.running}>
                {versions.length === 0 ? <MenuItem value={DEFAULT_DSH_VERSION}>{DEFAULT_DSH_VERSION}</MenuItem> : versions.map((item) => <MenuItem key={item.version} value={item.version}>{item.version}{item.isDefault ? ` · ${t("launcher.default")}` : ""}</MenuItem>)}
              </Select>
            </FormControl>
            <Tooltip title={t("launcher.manageVersions")}><span><IconButton className="version-button" aria-label={t("launcher.manageVersions")} onClick={() => setVersionDialog(true)} disabled={Boolean(busy) || status.running}><AddRounded /></IconButton></span></Tooltip>
          </Stack>

          <Button className="workspace-button" variant="outlined" startIcon={<FolderOpenRounded />} onClick={() => void chooseWorkspace()} disabled={Boolean(busy) || status.running}>
            <span>{workspace ? displayWorkspacePath(workspace) : t("launcher.chooseWorkspace")}</span>
          </Button>
        </section>

        <section className="launch-zone" aria-label={t("launcher.runControl")}>
          <Box className="ambient-visual" aria-hidden="true">
            <img className="ambient-brand-mark" src="/deepseek-mark.svg" alt="" />
            <span className="ambient-name">DeepSeek Harness</span>
            <span className="ambient-copy">{t("brand.tagline")}</span>
          </Box>
          <div className={`launch-actions ${showRuntimeActions ? "is-split" : "is-single"}`}>
            <button className={`launch-control ${launching ? "working" : ""}`} onClick={() => void handleToggle()} disabled={busy === "setup" || showRuntimeActions} tabIndex={showRuntimeActions ? -1 : undefined} aria-hidden={showRuntimeActions} aria-label={t("launcher.startAria")}>
              <span className="control-ring" aria-hidden="true" />
              <span className="control-icon" aria-hidden="true">
                {launching ? <CircularProgress size={34} thickness={3.5} color="inherit" /> : <PowerSettingsNewRounded />}
              </span>
              <span className="control-label">{launching ? t("launcher.starting") : t("launcher.start")}</span>
            </button>
            <div className="split-controls" aria-hidden={!showRuntimeActions}>
              <button className="split-control stop-control" onClick={() => void handleToggle()} disabled={!showRuntimeActions || restarting || stopping} tabIndex={showRuntimeActions ? undefined : -1} aria-label={t("launcher.stopAria")}>
                <span className="split-icon" aria-hidden="true">{stopping ? <CircularProgress size={22} thickness={4} color="inherit" /> : <StopRounded />}</span>
                <span>{stopping ? t("launcher.stopping") : t("launcher.stop")}</span>
              </button>
              <button className="split-control restart-control" onClick={() => void handleRestart()} disabled={!showRuntimeActions || restarting || stopping} tabIndex={showRuntimeActions ? undefined : -1} aria-label={t("launcher.restartAria")}>
                <span className={`split-icon ${restarting ? "is-spinning" : ""}`} aria-hidden="true">{restarting ? <CircularProgress size={22} thickness={4} color="inherit" /> : <RestartAltRounded />}</span>
                <span>{restarting ? t("launcher.restarting") : t("launcher.restart")}</span>
              </button>
            </div>
          </div>
          {status.running && status.endpoint && <Button className="open-dsh" size="small" endIcon={<ArrowOutwardRounded />} onClick={() => void openUrl(status.endpoint!)}>{t("launcher.openDsh")}</Button>}
        </section>
      </main>

      <Box className="status-block" aria-live="polite">
        <span className={`live-dot ${status.ready ? "ready" : ""}`} />
        <Typography variant="body2">{t(lastMessageKey, { version: status.dshVersion ?? "" })}</Typography>
      </Box>

      <Dialog open={versionDialog} onClose={() => !busy && setVersionDialog(false)} fullWidth maxWidth="xs">
        <DialogTitle>{t("versions.title")}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} pt={0.5}>
            <Typography variant="body2" color="text.secondary">{t("versions.isolationHint")}</Typography>
            {error && <Alert severity="error" onClose={() => setError(null)}>{error}</Alert>}
            <Button variant="outlined" startIcon={busy === "check" ? <CircularProgress size={18} /> : <RefreshRounded />} onClick={() => void checkRemoteVersions()} disabled={Boolean(busy)}>
              {busy === "check" ? t("versions.checking") : t("versions.checkOnline")}
            </Button>
            {versions.map((item) => (
              <Stack key={item.version} direction="row" alignItems="center" justifyContent="space-between">
                <Typography fontWeight={650}>{item.version}{item.isDefault ? ` · ${t("launcher.default")}` : ""}</Typography>
                <Stack direction="row" spacing={0.5}>
                  {!item.isDefault && <Tooltip title={t("versions.setDefault")}><span><IconButton size="small" aria-label={t("versions.setDefaultAria", { version: item.version })} onClick={() => void setDefaultVersion(item.version)} disabled={Boolean(busy)}><CheckRounded fontSize="small" /></IconButton></span></Tooltip>}
                  <IconButton size="small" aria-label={t("versions.removeAria", { version: item.version })} onClick={() => void removeVersion(item.version)} disabled={item.isDefault || item.isRunning || Boolean(busy)}><DeleteOutlineRounded fontSize="small" /></IconButton>
                </Stack>
              </Stack>
            ))}
            {remoteVersions.length > 0 ? (
              <TextField select label={t("versions.onlineVersion")} value={newVersion} onChange={(event) => setNewVersion(event.target.value)} size="small">
                {remoteVersions.slice(0, 24).map((version, index) => <MenuItem key={version} value={version}>{version}{index === 0 ? ` · ${t("versions.latest")}` : ""}{versions.some((item) => item.version === version && item.installed) ? ` · ${t("versions.installed")}` : ""}</MenuItem>)}
              </TextField>
            ) : (
              <TextField label={t("versions.installVersion")} value={newVersion} onChange={(event) => setNewVersion(event.target.value)} placeholder="0.1.6-alpha.2" size="small" />
            )}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setVersionDialog(false)} disabled={Boolean(busy)}>{t("versions.cancel")}</Button>
          <Button variant="contained" startIcon={<DownloadRounded />} onClick={() => void installVersion()} disabled={!newVersion.trim() || Boolean(busy)}>{busy === "install" ? t("versions.installing") : t("versions.install")}</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
