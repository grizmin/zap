# Working on the grizmin/zap fork

How this fork is set up and how to keep it running. This file lives only on `dev`;
never include it in a PR branch.

## Remotes and branches

| Remote   | URL                                   | Used for                          |
|----------|---------------------------------------|-----------------------------------|
| `origin` | https://github.com/grizmin/zap        | everything you push               |
| `zap`    | https://github.com/zerx-lab/zap       | new Zap changes, target of PRs    |

| Branch    | `git pull` from | Purpose                                                      |
|-----------|-----------------|--------------------------------------------------------------|
| `main`    | `zap/main`      | clean copy of Zap; starting point for every fix branch       |
| `fix/...` | `origin`        | one branch = one PR to zerx-lab/zap                          |
| `dev`     | `origin/dev`    | `zap/main` + all your fixes + local build setup; build this  |

`dev` is the default branch of the fork on GitHub.

Rules:
- PR branches start from `main`, never from `dev`.
- Local-only things (signing certs, this file, `.gitignore` additions) go on `dev` only.
- Update `dev` with merges, never rebase it (the fork is public).

## Commit identity

Set globally on this machine (`git config --global`): Konstantin Krastev <grizmin@gmail.com>.
On a new machine:

```powershell
git config --global user.name  "Konstantin Krastev"
git config --global user.email "grizmin@gmail.com"
```

## Pull in new Zap changes

```powershell
git switch main; git pull                 # main follows zerx-lab/zap
git push origin main                      # optional: keep the fork's main current
git switch dev; git merge main; git push  # bring them into dev
```

Fixes that zerx-lab has already merged drop out of `git diff main dev` on their own.

## Make a fix and send a PR

```powershell
git switch main; git pull
git switch -c fix/<short-name>
# ...edit, test...
git add <files>; git commit
git push -u origin fix/<short-name>
gh pr create --repo zerx-lab/zap --base main --head grizmin:fix/<short-name>

# use it right away, without waiting for the merge
git switch dev; git merge --no-ff fix/<short-name>; git push
```

After zerx-lab merges the PR:

```powershell
git branch -d fix/<short-name>; git push origin --delete fix/<short-name>
```

PR tips:
- One fix per PR. Small PRs get merged; big ones sit.
- Skip the Warp leftovers in `.github/pull_request_template.md` (server API, Agent Mode,
  Notion links). Keep Description, Testing and a `CHANGELOG-BUG-FIX:` line.
- zerx-lab merges in batches, sometimes months apart. That's why `dev` exists.

## Port a fix from upstream Warp

One-time setup:

```powershell
git remote add warp https://github.com/warpdotdev/warp.git
git fetch warp master
```

Per fix:

```powershell
git switch main; git pull
git switch -c fix/<short-name>
git cherry-pick -x <warp-commit>   # -x records the original commit for credit
# resolve conflicts, build, test, then push and open a PR as above
```

Zap forked from Warp at `c325d146a` (2026-04-28). Useful commands:

```powershell
git log --oneline c325d146a..warp/master -- crates/warp_terminal   # upstream changes in an area
git show <commit>                                                  # read one change
```

Zap removed Warp's cloud code (GraphQL, sync, sign-in, teams, Warp AI). Anything that
depends on it has to be rewritten, not cherry-picked. Terminal and editor fixes usually
port well.

## Build the Windows installer

Prerequisites (one-time):
- Strawberry Perl on `PATH` (only needed if OpenSSL is built from source, see below)
- Inno Setup 7, installed per-user in `%LOCALAPPDATA%\Programs\Inno Setup 7`
- `cargo install --locked cargo-about@0.8.4` (generates the third-party license file)

Build from `dev`:

```powershell
git switch dev
$env:PATH = "$env:LOCALAPPDATA\Programs\Inno Setup 7;$env:PATH"
$env:GIT_RELEASE_TAG = (git describe --tags --abbrev=0)   # version shown in the app
.\script\windows\bundle.ps1 -CHANNEL oss -ARCH x64
```

- Takes about 12 minutes (release build with LTO).
- Output: `script\windows\Output\ZapSetup.exe`; the exe is in
  `target\x86_64-pc-windows-msvc\rlto\zap-oss.exe`.
- `-SKIP_BUILD_BINARY` repackages without recompiling (about 1.5 minutes).
- Don't copy `target\debug\zap-oss.exe` to other machines: it's a debug build, opens a
  console window and lacks the DLLs the installer ships.

## Sign the build

Files in `script\windows\signing\`:

| File                     | What                                   | In git |
|--------------------------|----------------------------------------|--------|
| `zap-local-root.cer`     | issuer; trust this on other machines   | yes    |
| `zap-local-signing.cer`  | public signing certificate             | yes    |
| `zap-local-signing.pfx`  | signing key                            | no     |
| `zap-local-root.pfx`     | issuer key (can create new certs)      | no     |
| `pfx.password`           | password for both `.pfx` files         | no     |

Back up the three ignored files. Without them you can't sign, and a new issuer means
re-trusting it on every machine. Never put them in a shared or synced folder.

Sign the exe first, then package (Inno signs the setup and the uninstaller):

```powershell
$pfx = (Resolve-Path script\windows\signing\zap-local-signing.pfx).Path
$pw  = Get-Content script\windows\signing\pfx.password -Raw
$st  = "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"
$ts  = 'http://timestamp.digicert.com'

& $st sign /fd SHA256 /f $pfx /p $pw /tr $ts /td SHA256 /d "Zap" `
  target\x86_64-pc-windows-msvc\rlto\zap-oss.exe

.\script\windows\bundle.ps1 -CHANNEL oss -ARCH x64 -SKIP_BUILD_BINARY `
  -SIGN_TOOL_CMD "`$q$st`$q sign /fd SHA256 /f `$q$pfx`$q /p $pw /tr $ts /td SHA256 `$f"
```

Check: `Get-AuthenticodeSignature script\windows\Output\ZapSetup.exe`. On a machine that
doesn't trust the issuer the status is `UnknownError`; that's expected.

Certificates expire: signing cert 2029, issuer 2031. The timestamp keeps already-signed
files valid after that.

## Install on another machine (no admin needed)

Copy `ZapSetup.exe` and `zap-local-root.cer`, then:

```powershell
Import-Certificate -FilePath .\zap-local-root.cer -CertStoreLocation Cert:\CurrentUser\Root
Unblock-File .\ZapSetup.exe
.\ZapSetup.exe
```

- The certificate import shows a security prompt; answer Yes. Company policy may block it;
  the app still installs unsigned.
- `Unblock-File` is still needed: SmartScreen ignores certificates you trust yourself.
- The installer installs per user into `%LOCALAPPDATA%\Programs\Zap`.
- If it won't start: check Event Viewer > Windows Logs > Application, and AppLocker/WDAC:

  ```powershell
  Get-WinEvent -LogName 'Microsoft-Windows-AppLocker/EXE and DLL' -MaxEvents 10
  Get-WinEvent -LogName 'Microsoft-Windows-CodeIntegrity/Operational' -MaxEvents 10
  ```

## Prebuilt OpenSSL (faster builds)

`zap_sftp` asks for a vendored OpenSSL, which compiles OpenSSL with Perl on every clean
build. This machine skips that via `%USERPROFILE%\.cargo\config.toml`:

```toml
[env]
X86_64_PC_WINDOWS_MSVC_OPENSSL_NO_VENDOR = "1"
X86_64_PC_WINDOWS_MSVC_OPENSSL_DIR = 'C:\Users\grizm\.local\openssl-3.6.3-x64-static'
X86_64_PC_WINDOWS_MSVC_OPENSSL_STATIC = "1"
```

- Applies only to Windows MSVC builds; other targets still use the vendored copy.
- The OpenSSL in that folder was copied from an earlier vendored build in `target\`.
- On a new machine: build once without this config, copy
  `target\debug\build\openssl-sys-*\out\openssl-build\install` to that folder, then add
  the config. Or delete the config to go back to building from source.
- The copy stays at 3.6.3; replace it when OpenSSL ships security fixes.
