use super::register_uri_scheme;
use anyhow::{Context as _, Result};
use gpui::{AppContext as _, AsyncApp, Context, PromptLevel, Window, actions};
use release_channel::ReleaseChannel;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use util::ResultExt;
use workspace::notifications::{DetachAndPromptErr, NotificationId};
use workspace::{Toast, Workspace};

actions!(
    cli,
    [
        /// Don Editor CLI 도구를 시스템 PATH에 설치합니다.
        InstallCliBinary,
    ]
);

/// 원본 Gram CLI를 덮어쓰지 않고 Don Editor 전용 심볼릭 링크를 설치합니다.
async fn install_script(cx: &AsyncApp) -> Result<PathBuf> {
    let cli_path = cx.update(|cx| cx.path_for_auxiliary_executable("cli"))??;
    let link_path = Path::new("/usr/local/bin/doneditor");
    let bin_dir_path = link_path.parent().unwrap();

    // Don't re-create symlink if it points to the same CLI binary.
    if smol::fs::read_link(link_path).await.ok().as_ref() == Some(&cli_path) {
        return Ok(link_path.into());
    }

    // If the symlink is not there or is outdated, first try replacing it
    // without escalating.
    smol::fs::remove_file(link_path).await.log_err();
    if smol::fs::unix::symlink(&cli_path, link_path).await.log_err().is_some() {
        return Ok(link_path.into());
    }

    // The symlink could not be created, so use osascript with admin privileges
    // to create it.
    let status = smol::process::Command::new("/usr/bin/osascript")
        .args([
            "-e",
            &format!(
                "do shell script \" \
                    mkdir -p \'{}\' && \
                    ln -sf \'{}\' \'{}\' \
                \" with administrator privileges",
                bin_dir_path.to_string_lossy(),
                cli_path.to_string_lossy(),
                link_path.to_string_lossy(),
            ),
        ])
        .stdout(smol::process::Stdio::inherit())
        .stderr(smol::process::Stdio::inherit())
        .output()
        .await?
        .status;
    anyhow::ensure!(status.success(), "error running osascript");
    Ok(link_path.into())
}

/// 플랫폼별 CLI 설치 방법을 안내하고 설치 결과를 현재 표시 언어로 알립니다.
pub fn install_cli_binary(window: &mut Window, cx: &mut Context<Workspace>) {
    cx.spawn_in(window, async move |workspace, cx| {
        if cfg!(any(target_os = "linux", target_os = "freebsd")) {
            let prompt = cx.prompt(
                PromptLevel::Warning,
                i18n::t!("cli.install.available"),
                Some(i18n::t!("cli.install.linux_detail")),
                &[i18n::t!("common.ok")],
            );
            cx.background_spawn(prompt).detach();
            return Ok(());
        }
        let path = install_script(cx.deref()).await.context("error creating CLI symlink")?;

        workspace.update_in(cx, |workspace, _, cx| {
            struct InstalledGramCli;

            workspace.show_toast(
                Toast::new(
                    NotificationId::unique::<InstalledGramCli>(),
                    i18n::t!(
                        "cli.install.success",
                        path = path.to_string_lossy(),
                        application = ReleaseChannel::global(cx).display_name()
                    ),
                ),
                cx,
            )
        })?;
        register_uri_scheme(cx).await.log_err();
        Ok(())
    })
    .detach_and_prompt_err(i18n::t!("cli.install.failed"), window, cx, |_, _, _| None);
}
