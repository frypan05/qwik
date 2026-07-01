use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result};

const LS_GITHUB_REPO: &str = "frypan05/qwik-route-ls";

struct QwikExtension {
    cached_binary_path: Option<String>,
}

impl QwikExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // Prefer a binary already on PATH (e.g. installed for local dev/testing).
        if let Some(path) = worktree.which("qwik-route-ls") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            LS_GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let arch_str = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X86 => "x86",
            zed::Architecture::X8664 => "x86_64",
        };
        let os_str = match platform {
            zed::Os::Mac => "apple-darwin",
            zed::Os::Linux => "unknown-linux-gnu",
            zed::Os::Windows => "pc-windows-msvc",
        };
        let ext = match platform {
            zed::Os::Windows => "zip",
            _ => "tar.gz",
        };
        let asset_name = format!("qwik-route-ls-{arch_str}-{os_str}.{ext}");

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no release asset found matching {asset_name:?}"))?;

        let version_dir = format!("qwik-route-ls-{}", release.version);
        let binary_path = format!(
            "{version_dir}/qwik-route-ls{}",
            match platform {
                zed::Os::Windows => ".exe",
                _ => "",
            }
        );

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                    _ => zed::DownloadedFileType::GzipTar,
                },
            )
            .map_err(|e| format!("failed to download qwik-route-ls: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            // Clean up old cached versions.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    if entry.file_name().to_str() != Some(version_dir.as_str())
                        && entry.file_name().to_string_lossy().starts_with("qwik-route-ls-")
                    {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for QwikExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let path = self.language_server_binary_path(language_server_id, worktree)?;
        Ok(zed::Command {
            command: path,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(QwikExtension);