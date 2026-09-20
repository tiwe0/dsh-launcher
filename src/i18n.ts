import i18n from "i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import { initReactI18next } from "react-i18next";

const resources = {
  zh: {
    translation: {
      language: {
        label: "界面语言",
        toggleToEnglish: "切换至英文",
        toggleToChinese: "Switch to Chinese",
      },
      window: {
        controls: "dsh launcher 窗口控制",
        minimize: "最小化",
        close: "关闭",
      },
      brand: {
        badge: "Harness 启动器",
        tagline: "万物皆插件。",
        openHarness: "打开 DeepSeek Harness 官网",
      },
      links: {
        openGitHub: "在 GitHub 打开项目",
      },
      settings: {
        open: "高级配置",
        title: "高级配置",
        back: "返回启动器",
        pageDescription: "管理 dsh launcher 的独立运行环境与持久化配置。",
        runtimeTitle: "DSH 运行环境",
        description: "自定义 DSH 的配置与 profile 存储目录。",
        chooseDshHome: "选择 DSH_HOME 目录",
        defaultPath: "默认：{{path}}",
        appliesNextLaunch: "新的目录将在下次启动 DSH 时生效。",
        saved: "DSH_HOME 已保存",
        restoreDefault: "恢复默认",
        save: "保存",
      },
      launcher: {
        config: "启动配置",
        title: "启动 Harness",
        subtitle: "选择独立版本与本次运行目录。",
        nodeVersion: "Node 版本",
        dshVersion: "DSH 版本",
        recommended: "推荐",
        onDemand: "按需下载",
        default: "默认",
        manageVersions: "管理 DSH 版本",
        chooseWorkspace: "选择工作目录",
        chooseWorkspaceDialog: "选择 DSH 工作目录",
        runControl: "运行控制",
        startAria: "启动 DeepSeek Harness",
        stopAria: "关闭 DeepSeek Harness",
        restartAria: "重启 DeepSeek Harness",
        start: "启动",
        stop: "关闭",
        restart: "重启",
        starting: "启动中",
        stopping: "关闭中",
        restarting: "重启中",
        openDsh: "打开 DSH",
      },
      status: {
        notCreated: "隔离运行时尚未创建",
        ready: "隔离运行时已就绪",
        running: "DSH {{version}} 正在运行",
        started: "DeepSeek Harness 已启动",
        stopped: "DeepSeek Harness 已关闭",
        waiting: "正在等待 DSH Web 服务就绪…",
        stopping: "正在关闭 DeepSeek Harness…",
        preparingRuntime: "首次启动：正在准备独立 Node 与 DSH…",
        preparingVersionCheck: "正在准备版本检测环境…",
        starting: "正在启动 DeepSeek Harness…",
        restarting: "正在重启 DeepSeek Harness…",
      },
      versions: {
        title: "管理 DSH 版本",
        isolationHint: "每个版本安装在独立目录，不会覆盖其他版本。",
        checking: "检测中…",
        checkOnline: "检测在线版本",
        setDefault: "设为默认",
        setDefaultAria: "将 DSH {{version}} 设为默认",
        removeAria: "移除 DSH {{version}}",
        onlineVersion: "在线版本",
        installVersion: "安装版本",
        latest: "最新",
        installed: "已安装",
        cancel: "取消",
        installing: "安装中…",
        install: "下载安装",
      },
      errors: {
        selectWorkspace: "请先选择工作目录",
        runtime: "运行时发生错误",
      },
    },
  },
  en: {
    translation: {
      language: {
        label: "Interface language",
        toggleToEnglish: "Switch to English",
        toggleToChinese: "Switch to Chinese",
      },
      window: {
        controls: "dsh launcher window controls",
        minimize: "Minimize",
        close: "Close",
      },
      brand: {
        badge: "Harness Launcher",
        tagline: "Everything is a plugin.",
        openHarness: "Open the DeepSeek Harness website",
      },
      links: {
        openGitHub: "Open the project on GitHub",
      },
      settings: {
        open: "Advanced settings",
        title: "Advanced settings",
        back: "Back to launcher",
        pageDescription: "Manage the isolated runtime and persistent settings used by dsh launcher.",
        runtimeTitle: "DSH runtime",
        description: "Choose where DSH stores configuration and profiles.",
        chooseDshHome: "Choose the DSH_HOME folder",
        defaultPath: "Default: {{path}}",
        appliesNextLaunch: "The new folder will be used the next time DSH starts.",
        saved: "DSH_HOME saved",
        restoreDefault: "Restore default",
        save: "Save",
      },
      launcher: {
        config: "Launch configuration",
        title: "Launch Harness",
        subtitle: "Choose isolated versions and a workspace for this session.",
        nodeVersion: "Node version",
        dshVersion: "DSH version",
        recommended: "Recommended",
        onDemand: "Download on demand",
        default: "Default",
        manageVersions: "Manage DSH versions",
        chooseWorkspace: "Choose workspace",
        chooseWorkspaceDialog: "Choose DSH workspace",
        runControl: "Runtime controls",
        startAria: "Launch DeepSeek Harness",
        stopAria: "Stop DeepSeek Harness",
        restartAria: "Restart DeepSeek Harness",
        start: "Launch",
        stop: "Stop",
        restart: "Restart",
        starting: "Launching",
        stopping: "Stopping",
        restarting: "Restarting",
        openDsh: "Open DSH",
      },
      status: {
        notCreated: "Isolated runtime has not been created",
        ready: "Isolated runtime is ready",
        running: "DSH {{version}} is running",
        started: "DeepSeek Harness launched",
        stopped: "DeepSeek Harness stopped",
        waiting: "Waiting for the DSH Web service…",
        stopping: "Stopping DeepSeek Harness…",
        preparingRuntime: "First launch: preparing isolated Node and DSH…",
        preparingVersionCheck: "Preparing the version check environment…",
        starting: "Launching DeepSeek Harness…",
        restarting: "Restarting DeepSeek Harness…",
      },
      versions: {
        title: "Manage DSH versions",
        isolationHint: "Each version is installed in its own directory and never overwrites another version.",
        checking: "Checking…",
        checkOnline: "Check online versions",
        setDefault: "Set as default",
        setDefaultAria: "Set DSH {{version}} as default",
        removeAria: "Remove DSH {{version}}",
        onlineVersion: "Online version",
        installVersion: "Version to install",
        latest: "Latest",
        installed: "Installed",
        cancel: "Cancel",
        installing: "Installing…",
        install: "Download and install",
      },
      errors: {
        selectWorkspace: "Choose a workspace first",
        runtime: "A runtime error occurred",
      },
    },
  },
} as const;

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "zh",
    supportedLngs: ["zh", "en"],
    load: "languageOnly",
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
      lookupLocalStorage: "dsh-launcher-language",
    },
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });

const syncDocumentLanguage = (language: string) => {
  document.documentElement.lang = language.startsWith("zh") ? "zh-CN" : "en";
};

syncDocumentLanguage(i18n.resolvedLanguage ?? i18n.language ?? "zh");
i18n.on("languageChanged", syncDocumentLanguage);

export default i18n;
