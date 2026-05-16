use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ShortcutState {
    pub installed: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryPointState {
    pub silent_shortcut: ShortcutState,
    pub management_shortcut: ShortcutState,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallActionResult {
    pub status: String,
    pub message: String,
    pub silent_shortcut: ShortcutState,
    pub management_shortcut: ShortcutState,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    #[serde(default)]
    pub remove_owned_data: bool,
}

impl ShortcutState {
    fn missing(path: Option<PathBuf>) -> Self {
        Self {
            installed: false,
            path: path.map(|path| path.to_string_lossy().to_string()),
        }
    }

    fn from_candidates(candidates: Vec<PathBuf>) -> Self {
        if let Some(path) = candidates.iter().find(|path| path.exists()) {
            return Self {
                installed: true,
                path: Some(path.to_string_lossy().to_string()),
            };
        }
        Self::missing(candidates.into_iter().next())
    }
}

pub fn inspect_entrypoints() -> EntryPointState {
    let desktop = desktop_dir();
    EntryPointState {
        silent_shortcut: ShortcutState::from_candidates(shortcut_candidates(&desktop, "Codex++")),
        management_shortcut: ShortcutState::from_candidates(shortcut_candidates(
            &desktop,
            "Codex++ 管理工具",
        )),
    }
}

pub fn install_entrypoints() -> InstallActionResult {
    skipped("安装入口尚未迁移到 Tauri；Task 8 将接入真实安装器。")
}

pub fn uninstall_entrypoints(options: InstallOptions) -> InstallActionResult {
    let suffix = if options.remove_owned_data {
        "已请求移除托管数据，但当前实现尚未执行。"
    } else {
        "未请求移除托管数据。"
    };
    skipped(&format!(
        "卸载入口尚未迁移到 Tauri；Task 8 将接入真实卸载器。{suffix}"
    ))
}

pub fn repair_shortcuts() -> InstallActionResult {
    skipped("快捷方式修复尚未迁移到 Tauri；Task 8 将接入真实修复逻辑。")
}

fn skipped(message: &str) -> InstallActionResult {
    let state = inspect_entrypoints();
    InstallActionResult {
        status: "not_implemented".to_string(),
        message: message.to_string(),
        silent_shortcut: state.silent_shortcut,
        management_shortcut: state.management_shortcut,
    }
}

fn desktop_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            return Some(PathBuf::from(user_profile).join("Desktop"));
        }
    }
    directories::UserDirs::new().and_then(|dirs| dirs.desktop_dir().map(PathBuf::from))
}

fn shortcut_candidates(desktop: &Option<PathBuf>, name: &str) -> Vec<PathBuf> {
    let Some(desktop) = desktop else {
        return Vec::new();
    };
    if cfg!(windows) {
        vec![desktop.join(format!("{name}.lnk"))]
    } else if cfg!(target_os = "macos") {
        vec![desktop.join(format!("{name}.app"))]
    } else {
        vec![desktop.join(format!("{name}.desktop"))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_stub_reports_not_implemented_with_shortcut_state() {
        let result = install_entrypoints();

        assert_eq!(result.status, "not_implemented");
        assert!(!result.silent_shortcut.installed);
        assert!(!result.management_shortcut.installed);
    }

    #[test]
    fn shortcut_state_detects_existing_candidate() {
        let temp = std::env::temp_dir().join(format!(
            "codex-plus-shortcut-test-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&temp);
        let shortcut = temp.join("Codex++ 管理工具.lnk");
        std::fs::write(&shortcut, "shortcut").unwrap();

        let state = ShortcutState::from_candidates(vec![shortcut.clone()]);

        assert!(state.installed);
        assert_eq!(state.path.as_deref(), Some(shortcut.to_string_lossy().as_ref()));
        let _ = std::fs::remove_file(shortcut);
        let _ = std::fs::remove_dir(temp);
    }

    #[test]
    fn uninstall_message_mentions_owned_data_request() {
        let result = uninstall_entrypoints(InstallOptions {
            remove_owned_data: true,
        });

        assert_eq!(result.status, "not_implemented");
        assert!(result.message.contains("移除托管数据"));
    }
}
