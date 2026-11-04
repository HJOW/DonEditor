use anyhow::Context as _;

use git::repository::{Remote, RemoteCommandOutput};
use linkify::{LinkFinder, LinkKind};
use ui::SharedString;
use util::ResultExt as _;

#[derive(Clone)]
pub enum RemoteAction {
    Fetch(Option<Remote>),
    Pull(Remote),
    Push(SharedString, Remote),
}

impl RemoteAction {
    pub fn name(&self) -> &'static str {
        match self {
            RemoteAction::Fetch(_) => "fetch",
            RemoteAction::Pull(_) => "pull",
            RemoteAction::Push(_, _) => "push",
        }
    }
}

pub enum SuccessStyle {
    Toast,
    ToastWithLog { output: RemoteCommandOutput },
    PushPrLink { text: String, link: String },
}

pub struct SuccessMessage {
    pub message: String,
    pub style: SuccessStyle,
}

pub fn format_output(action: &RemoteAction, output: RemoteCommandOutput) -> SuccessMessage {
    match action {
        RemoteAction::Fetch(remote) => {
            if output.stderr.is_empty() {
                SuccessMessage {
                    message: i18n::t!("ui.text.fetch_already_up_to_date").into(),
                    style: SuccessStyle::Toast,
                }
            } else {
                let message = match remote {
                    Some(remote) => i18n::t!("ui.text.synchronized_with", value0 = remote.name),
                    None => i18n::t!("ui.text.synchronized_with_remotes").into(),
                };
                SuccessMessage {
                    message,
                    style: SuccessStyle::ToastWithLog { output },
                }
            }
        }
        RemoteAction::Pull(remote_ref) => {
            let get_changes = |output: &RemoteCommandOutput| -> anyhow::Result<u32> {
                let last_line = output
                    .stdout
                    .lines()
                    .last()
                    .context("Failed to get last line of output")?
                    .trim();

                let files_changed = last_line
                    .split_whitespace()
                    .next()
                    .context("Failed to get first word of last line")?
                    .parse()?;

                Ok(files_changed)
            };
            if output.stdout.ends_with("Already up to date.\n") {
                SuccessMessage {
                    message: i18n::t!("ui.text.pull_already_up_to_date").into(),
                    style: SuccessStyle::Toast,
                }
            } else if output.stdout.starts_with("Updating") {
                let files_changed = get_changes(&output).log_err();
                let message = if let Some(files_changed) = files_changed {
                    i18n::t!(
                        "ui.text.received_file_change_from",
                        value0 = files_changed,
                        value2 = remote_ref.name
                    )
                } else {
                    i18n::t!("ui.text.fast_forwarded_from", value0 = remote_ref.name)
                };
                SuccessMessage {
                    message,
                    style: SuccessStyle::ToastWithLog { output },
                }
            } else if output.stdout.starts_with("Merge") {
                let files_changed = get_changes(&output).log_err();
                let message = if let Some(files_changed) = files_changed {
                    i18n::t!(
                        "ui.text.merged_file_change_from",
                        value0 = files_changed,
                        value2 = remote_ref.name
                    )
                } else {
                    i18n::t!("ui.text.merged_from", value0 = remote_ref.name)
                };
                SuccessMessage {
                    message,
                    style: SuccessStyle::ToastWithLog { output },
                }
            } else if output.stdout.contains("Successfully rebased") {
                SuccessMessage {
                    message: i18n::t!("ui.text.successfully_rebased_from", value0 = remote_ref.name),
                    style: SuccessStyle::ToastWithLog { output },
                }
            } else {
                SuccessMessage {
                    message: i18n::t!("ui.text.successfully_pulled_from", value0 = remote_ref.name),
                    style: SuccessStyle::ToastWithLog { output },
                }
            }
        }
        RemoteAction::Push(branch_name, remote_ref) => {
            let message = if output.stderr.ends_with("Everything up-to-date\n") {
                i18n::t!("ui.text.push_everything_is_up_to_date").to_string()
            } else {
                i18n::t!("git.pushed_to", branch = branch_name, remote = remote_ref.name)
            };

            let style = if output.stderr.ends_with("Everything up-to-date\n") {
                Some(SuccessStyle::Toast)
            } else if output.stderr.contains("\nremote: ") {
                let pr_hints = [
                    ("Create a pull request", i18n::t!("ui.text.create_pull_request")), // GitHub
                    ("Create pull request", i18n::t!("ui.text.create_pull_request")),   // Bitbucket
                    ("create a merge request", i18n::t!("ui.text.create_merge_request")), // GitLab
                    ("View merge request", i18n::t!("ui.text.view_merge_request_c630a828")), // GitLab
                ];
                pr_hints
                    .iter()
                    .find(|(indicator, _)| output.stderr.contains(indicator))
                    .and_then(|(_, mapped)| {
                        let finder = LinkFinder::new();
                        finder
                            .links(&output.stderr)
                            .filter(|link| *link.kind() == LinkKind::Url)
                            .filter(|link| !link.as_str().contains("www.openssh.org"))
                            .map(|link| link.start()..link.end())
                            .next()
                            .map(|link| SuccessStyle::PushPrLink {
                                text: mapped.to_string(),
                                link: output.stderr[link].to_string(),
                            })
                    })
            } else {
                None
            };
            SuccessMessage {
                message,
                style: style.unwrap_or(SuccessStyle::ToastWithLog { output }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    /// 한국어에서도 Git의 영어 프로토콜은 그대로 해석하고 사용자에게 보이는 결과만 번역합니다.
    #[test]
    fn korean_output_preserves_git_protocol_and_user_values() {
        i18n::with_language(i18n::Language::Korean, || {
            let remote = Remote { name: "Save".into() };
            let action = RemoteAction::Push("Create".into(), remote.clone());
            let output = RemoteCommandOutput {
                stdout: String::new(),
                stderr: "\nremote: Create a pull request at https://example.com/pull/1\n".into(),
            };
            let result = format_output(&action, output);
            assert_eq!(result.message, "Create을(를) Save에 보냈습니다");
            let SuccessStyle::PushPrLink { text, link } = result.style else {
                panic!("영어 Git 출력에서 풀 리퀘스트 링크를 찾지 못했습니다");
            };
            assert_eq!(text, "풀 리퀘스트 만들기");
            assert_eq!(link, "https://example.com/pull/1");

            let result = format_output(
                &RemoteAction::Pull(remote),
                RemoteCommandOutput {
                    stdout: "Successfully rebased and updated refs/heads/Create.\n".into(),
                    stderr: String::new(),
                },
            );
            assert_eq!(result.message, "Save에서 리베이스했습니다");
        });
    }

    #[test]
    fn test_push_new_branch_pull_request() {
        let action = RemoteAction::Push(
            SharedString::new("test_branch"),
            Remote {
                name: SharedString::new("test_remote"),
            },
        );

        let output = RemoteCommandOutput {
            stdout: String::new(),
            stderr: indoc! { "
                Total 0 (delta 0), reused 0 (delta 0), pack-reused 0 (from 0)
                remote:
                remote: Create a pull request for 'test' on GitHub by visiting:
                remote:      https://example.com/test/test/pull/new/test
                remote:
                To example.com:test/test.git
                 * [new branch]      test -> test
                "}
            .to_string(),
        };

        let msg = format_output(&action, output);

        if let SuccessStyle::PushPrLink { text: hint, link } = &msg.style {
            assert_eq!(hint, "Create Pull Request");
            assert_eq!(link, "https://example.com/test/test/pull/new/test");
        } else {
            panic!("Expected PushPrLink variant");
        }
    }

    #[test]
    fn test_push_new_branch_merge_request() {
        let action = RemoteAction::Push(
            SharedString::new("test_branch"),
            Remote {
                name: SharedString::new("test_remote"),
            },
        );

        let output = RemoteCommandOutput {
            stdout: String::new(),
            stderr: indoc! {"
                Total 0 (delta 0), reused 0 (delta 0), pack-reused 0 (from 0)
                remote:
                remote: To create a merge request for test, visit:
                remote:   https://example.com/test/test/-/merge_requests/new?merge_request%5Bsource_branch%5D=test
                remote:
                To example.com:test/test.git
                 * [new branch]      test -> test
                "}
            .to_string(),
        };

        let msg = format_output(&action, output);

        if let SuccessStyle::PushPrLink { text, link } = &msg.style {
            assert_eq!(text, "Create Merge Request");
            assert_eq!(
                link,
                "https://example.com/test/test/-/merge_requests/new?merge_request%5Bsource_branch%5D=test"
            );
        } else {
            panic!("Expected PushPrLink variant");
        }
    }

    #[test]
    fn test_push_branch_existing_merge_request() {
        let action = RemoteAction::Push(
            SharedString::new("test_branch"),
            Remote {
                name: SharedString::new("test_remote"),
            },
        );

        let output = RemoteCommandOutput {
            stdout: String::new(),
            stderr: indoc! {"
                Total 0 (delta 0), reused 0 (delta 0), pack-reused 0 (from 0)
                remote:
                remote: View merge request for test:
                remote:    https://example.com/test/test/-/merge_requests/99999
                remote:
                To example.com:test/test.git
                    + 80bd3c83be...e03d499d2e test -> test
                "}
            .to_string(),
        };

        let msg = format_output(&action, output);

        if let SuccessStyle::PushPrLink { text, link } = &msg.style {
            assert_eq!(text, "View Merge Request");
            assert_eq!(link, "https://example.com/test/test/-/merge_requests/99999");
        } else {
            panic!("Expected PushPrLink variant");
        }
    }

    #[test]
    fn test_push_new_branch_no_link() {
        let action = RemoteAction::Push(
            SharedString::new("test_branch"),
            Remote {
                name: SharedString::new("test_remote"),
            },
        );

        let output = RemoteCommandOutput {
            stdout: String::new(),
            stderr: indoc! { "
                To http://example.com/test/test.git
                 * [new branch]      test -> test
                ",
            }
            .to_string(),
        };

        let msg = format_output(&action, output);

        if let SuccessStyle::ToastWithLog { output } = &msg.style {
            assert_eq!(
                output.stderr,
                "To http://example.com/test/test.git\n * [new branch]      test -> test\n"
            );
        } else {
            panic!("Expected ToastWithLog variant");
        }
    }
}
