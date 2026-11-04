//! Provides constructs for the Gram app version and release channel.

#![deny(missing_docs)]

use std::{env, str::FromStr, sync::LazyLock};

use gpui::{App, Global, SemanticVersion};

/// stable | dev
pub static RELEASE_CHANNEL_NAME: LazyLock<String> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        env::var("GRAM_RELEASE_CHANNEL")
            .unwrap_or_else(|_| include_str!("../../gram/RELEASE_CHANNEL").trim().to_string())
    } else {
        include_str!("../../gram/RELEASE_CHANNEL").trim().to_string()
    }
});

#[doc(hidden)]
pub static RELEASE_CHANNEL: LazyLock<ReleaseChannel> =
    LazyLock::new(|| match ReleaseChannel::from_str(&RELEASE_CHANNEL_NAME) {
        Ok(channel) => channel,
        _ => panic!("invalid release channel {}", *RELEASE_CHANNEL_NAME),
    });

/// The app identifier for the current release channel, Windows only.
// [DonEditor] 앱 식별자. 단일 인스턴스 뮤텍스와 명명 파이프 이름에 쓰이므로, 원본 Gram 과 달라야 동시에 실행할 수 있다.
#[cfg(target_os = "windows")]
pub fn app_identifier() -> &'static str {
    match *RELEASE_CHANNEL {
        ReleaseChannel::Dev => "DonEditor-Dev",
        ReleaseChannel::Stable => "DonEditor-Stable",
    }
}

/// The Git commit SHA that Gram was built at.
#[derive(Clone, Eq, Debug, PartialEq)]
pub struct AppCommitSha(String);

struct GlobalAppCommitSha(AppCommitSha);

impl Global for GlobalAppCommitSha {}

impl AppCommitSha {
    /// Creates a new [`AppCommitSha`].
    pub fn new(sha: String) -> Self {
        AppCommitSha(sha)
    }

    /// Returns the global [`AppCommitSha`], if one is set.
    pub fn try_global(cx: &App) -> Option<AppCommitSha> {
        cx.try_global::<GlobalAppCommitSha>().map(|sha| sha.0.clone())
    }

    /// Sets the global [`AppCommitSha`].
    pub fn set_global(sha: AppCommitSha, cx: &mut App) {
        cx.set_global(GlobalAppCommitSha(sha))
    }

    /// Returns the full commit SHA.
    pub fn full(&self) -> String {
        self.0.to_string()
    }

    /// Returns the short (7 character) commit SHA.
    pub fn short(&self) -> String {
        self.0.chars().take(7).collect()
    }
}

struct GlobalAppVersion(SemanticVersion);

impl Global for GlobalAppVersion {}

/// The version of Gram.
pub struct AppVersion;

impl AppVersion {
    /// Load the app version from env.
    pub fn load(pkg_version: &str) -> SemanticVersion {
        if let Ok(from_env) = env::var("GRAM_APP_VERSION") {
            from_env.parse().expect("invalid GRAM_APP_VERSION")
        } else {
            pkg_version.parse().expect("invalid version in Cargo.toml")
        }
    }

    /// Returns the global version number.
    pub fn global(cx: &App) -> SemanticVersion {
        if cx.has_global::<GlobalAppVersion>() {
            cx.global::<GlobalAppVersion>().0.clone()
        } else {
            SemanticVersion::new(0, 1, 0)
        }
    }
}

/// A Gram release channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ReleaseChannel {
    /// The development release channel.
    ///
    /// Used for local debug builds of Gram.
    #[default]
    Dev,

    /// The Stable release channel.
    Stable,
}

struct GlobalReleaseChannel(ReleaseChannel);

impl Global for GlobalReleaseChannel {}

/// Initializes the release channel.
pub fn init(app_version: SemanticVersion, cx: &mut App) {
    cx.set_global(GlobalAppVersion(app_version));
    cx.set_global(GlobalReleaseChannel(*RELEASE_CHANNEL))
}

/// Initializes the release channel for tests that rely on fake release channel.
pub fn init_test(app_version: SemanticVersion, release_channel: ReleaseChannel, cx: &mut App) {
    cx.set_global(GlobalAppVersion(app_version));
    cx.set_global(GlobalReleaseChannel(release_channel))
}

impl ReleaseChannel {
    /// Returns the global [`ReleaseChannel`].
    pub fn global(cx: &App) -> Self {
        cx.global::<GlobalReleaseChannel>().0
    }

    /// Returns the global [`ReleaseChannel`], if one is set.
    pub fn try_global(cx: &App) -> Option<Self> {
        cx.try_global::<GlobalReleaseChannel>().map(|channel| channel.0)
    }

    /// Returns whether we want to poll for updates for this [`ReleaseChannel`]
    pub fn poll_for_updates(&self) -> bool {
        !matches!(self, ReleaseChannel::Dev)
    }

    /// Returns the display name for this [`ReleaseChannel`].
    // [DonEditor] 화면에 표시되는 제품 이름 (채널별)
    pub fn display_name(&self) -> &'static str {
        match self {
            ReleaseChannel::Dev => "Don Editor Dev",
            ReleaseChannel::Stable => "Don Editor",
        }
    }

    /// Returns the programmatic name for this [`ReleaseChannel`].
    pub fn dev_name(&self) -> &'static str {
        match self {
            ReleaseChannel::Dev => "dev",
            ReleaseChannel::Stable => "stable",
        }
    }

    /// Returns the application ID that's used by Wayland as application ID
    /// and WM_CLASS on X11.
    /// This also has to match the bundle identifier for Gram on macOS.
    pub fn app_id(&self) -> &'static str {
        match self {
            // [DonEditor] macOS 번들 및 Linux 창 식별자를 원본과 분리한다.
            ReleaseChannel::Dev => "app.doneditor.DonEditor-Dev",
            ReleaseChannel::Stable => "app.doneditor.DonEditor",
        }
    }

    /// Returns the query parameter for this [`ReleaseChannel`].
    pub fn release_query_param(&self) -> Option<&'static str> {
        match self {
            Self::Dev => None,
            Self::Stable => None,
        }
    }
}

/// Error indicating that release channel string does not match any known release channel names.
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub struct InvalidReleaseChannel;

impl FromStr for ReleaseChannel {
    type Err = InvalidReleaseChannel;

    fn from_str(channel: &str) -> Result<Self, Self::Err> {
        Ok(match channel {
            "dev" => ReleaseChannel::Dev,
            "stable" => ReleaseChannel::Stable,
            _ => return Err(InvalidReleaseChannel),
        })
    }
}
