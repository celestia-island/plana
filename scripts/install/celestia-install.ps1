#Requires -Version 5.1
<#
.SYNOPSIS
    Celestia unified installer for Windows (WSL2 + native).

.DESCRIPTION
    Installs the four celestia project群 components from LOCAL source code:
      - entelecheia (scepter server) → built & run inside WSL2
      - evernight  (broker)          → built inside WSL2
      - scriptum   (TUI)              → built natively on Windows
      - shittim-chest (GUI/CLI shell) → built natively on Windows

    Idempotent: re-running skips work that is already done. Uses the local
    checkout at -SourceRoot (auto-detected from $PSScriptRoot by walking
    three levels up). NEVER clones from GitHub.

.PARAMETER SourceRoot       Override celestia source root (must contain entelecheia/Cargo.toml).
.PARAMETER SkipDocker       Skip Docker Engine installation inside WSL2.
.PARAMETER SkipBuild        Skip all cargo builds.
.PARAMETER SkipShortcuts    Skip Start Menu shortcut creation.
.PARAMETER NoMirror         Disable Docker registry mirror auto-configuration.
.PARAMETER Mirror           Override Docker registry mirror URL.
.PARAMETER Dev              Reserved (compat with legacy installer).
.PARAMETER Quick            Non-interactive; auto-accept all prompts.

.EXAMPLE
    .\celestia-install.ps1
    .\celestia-install.ps1 -Quick -SkipBuild
    .\celestia-install.ps1 -SourceRoot D:\src\celestia -NoMirror
#>

param(
    [string]$SourceRoot,
    [switch]$SkipDocker, [switch]$SkipBuild, [switch]$SkipShortcuts,
    [switch]$NoMirror,   [string]$Mirror,    [switch]$Dev, [switch]$Quick
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# ── Script-level state ──────────────────────────────────────────────────────
$script:DockerImages = @("pgvector/pgvector:pg18-bookworm")
$script:StateFile    = Join-Path $env:TEMP "celestia-install.state"
$script:InstallDir   = Join-Path $env:LOCALAPPDATA "Programs\celestia"
$script:StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\Celestia"
$script:ScepterPort  = "8424"
# Alpine minirootfs — lightweight WSL2 base (~3.3 MB). No existing WSL distro
# required; runs on any Windows machine with WSL2 enabled. Same model as
# Docker Desktop's managed WSL engine: self-contained, no host-side effects.
$script:BaseRootfsUrl = "https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-minirootfs-3.21.3-x86_64.tar.gz"
# WSL distro name is auto-generated per celestia isolation principle.
# NEVER hardcode Ubuntu-24.04 — see Resolve-CelestiaWslInstance.
$script:WSLDistro    = ""

# ── Helpers ─────────────────────────────────────────────────────────────────

function Write-Info  { param([string]$Msg) Write-Host "[INFO]  $Msg" -ForegroundColor Blue   }
function Write-Ok    { param([string]$Msg) Write-Host "[OK]    $Msg" -ForegroundColor Green  }
function Write-Warn  { param([string]$Msg) Write-Host "[WARN]  $Msg" -ForegroundColor Yellow }
function Write-Err   { param([string]$Msg) Write-Host "[ERROR] $Msg" -ForegroundColor Red    }
function Write-Step  { param([string]$Msg) Write-Host "`n==> $Msg" -ForegroundColor Cyan     }

function Confirm-Prompt {
    param([string]$Message, [bool]$Default = $true)
    if ($Quick) { return $Default }
    $yes = if ($Default) { "Y" } else { "y" }
    $no  = if ($Default) { "n" } else { "N" }
    $resp = Read-Host "$Message [$yes/$no]"
    if ([string]::IsNullOrWhiteSpace($resp)) { return $Default }
    return ($resp -match '^[Yy]')
}

# Run a command inside the configured WSL2 distro. Captures stdout+stderr.
function Invoke-WSL {
    param([Parameter(Mandatory)][string]$Command, [switch]$NoProfile)
    $wslArgs = @("-d", $script:WSLDistro)
    if ($NoProfile) { $wslArgs += @("--", "env", "-i", "bash", "-c", $Command) }
    else            { $wslArgs += @("--", "bash", "-lc", $Command) }
    return (& wsl @wslArgs 2>&1)
}

function Test-WSLCommand {
    param([string]$Command)
    return (-not [string]::IsNullOrWhiteSpace((Invoke-WSL -Command "command -v $Command" -NoProfile)))
}

# Convert a Windows path (d:\foo\bar) to its /mnt/<drive>/foo/bar WSL view.
function Convert-WinPathToWSL {
    param([Parameter(Mandatory)][string]$Path)
    $resolved = (Resolve-Path -LiteralPath $Path -ErrorAction Stop).Path
    if ($resolved -match '^([A-Za-z]):[\\/](.*)$') {
        return "/mnt/$($matches[1].ToLower())/$($matches[2] -replace '\\','/')"
    }
    return $resolved
}

# ── WSL2 instance isolation ─────────────────────────────────────────────────
# celestia MUST use its own dedicated WSL2 instance (celestia-XXX), never
# touching the user's Ubuntu-24.04 or any other pre-existing distro.
# Analogy: Docker Desktop uses its own WSL engine; podman-machine-default
# is self-contained. celestia follows the same principle.

function New-CelestiaInstanceName {
    $existingCelestia = & wsl --list --quiet 2>&1 | Where-Object { $_ -match '^celestia-\d{3}$' } | ForEach-Object { $_.Trim() }
    $maxAttempts = 100
    for ($i = 0; $i -lt $maxAttempts; $i++) {
        $id = Get-Random -Minimum 0 -Maximum 1000
        $name = "celestia-$($id.ToString('000'))"
        if ($existingCelestia -notcontains $name) { return $name }
    }
    Write-Err "Could not generate a unique celestia-XXX name after $maxAttempts attempts."
    Write-Err "Consider cleaning up old celestia instances: wsl --list --quiet | Select-String 'celestia-'"
    exit 1
}

function Resolve-CelestiaWslInstance {
    # 1. Check if CELESTIA_WSL_INSTANCE env var is set and the distro exists.
    $envInstance = [Environment]::GetEnvironmentVariable("CELESTIA_WSL_INSTANCE", "User")
    if ($envInstance) {
        $distros = & wsl --list --quiet 2>&1 | Where-Object { $_ -match '\S' } | ForEach-Object { $_.Trim() }
        if ($distros -contains $envInstance) {
            Write-Ok "Using existing celestia WSL instance: $envInstance (from CELESTIA_WSL_INSTANCE)"
            return $envInstance
        }
        Write-Warn "CELESTIA_WSL_INSTANCE=$envInstance no longer registered — generating new instance."
    }

    # 2. Scan for any existing celestia-XXX instance.
    $distros = & wsl --list --quiet 2>&1 | Where-Object { $_ -match '\S' } | ForEach-Object { $_.Trim() }
    $existingCelestia = $distros | Where-Object { $_ -match '^celestia-\d{3}$' }
    if ($existingCelestia) {
        $name = ($existingCelestia | Select-Object -First 1)
        Write-Ok "Found existing celestia WSL instance: $name"
        Persist-CelestiaInstanceEnv -Name $name
        return $name
    }

    # 3. No celestia instance exists — create one from a template distro.
    return (New-CelestiaWslInstance)
}

function Persist-CelestiaInstanceEnv {
    param([string]$Name)
    [Environment]::SetEnvironmentVariable("CELESTIA_WSL_INSTANCE", $Name, "User")
    $env:CELESTIA_WSL_INSTANCE = $Name
}

function New-CelestiaWslInstance {
    $name = New-CelestiaInstanceName
    Write-Step "Creating isolated WSL2 instance: $name"

    # Download Alpine minirootfs (~3.3 MB). No dependency on any pre-existing
    # WSL distro — works on a fresh Windows+WSL2 install with zero host setup.
    # Same model as Docker Desktop's self-contained WSL engine.
    $rootfsUrl = $script:BaseRootfsUrl
    $tempRootfs = Join-Path $env:TEMP "celestia-alpine-rootfs-$name.tar.gz"
    Write-Info "Downloading Alpine Linux base image (~3 MB)..."
    Write-Info "  $rootfsUrl"
    try {
        Invoke-WebRequest -Uri $rootfsUrl -OutFile $tempRootfs -UseBasicParsing -TimeoutSec 120
        $sizeMB = [math]::Round((Get-Item $tempRootfs).Length / 1MB, 1)
        Write-Ok "Downloaded: $sizeMB MB"
    } catch {
        Write-Err "Failed to download Alpine rootfs: $_"
        if (Test-Path $tempRootfs) { Remove-Item $tempRootfs -Force }
        exit 1
    }

    $installLoc = Join-Path $env:LOCALAPPDATA "celestia\$name"
    New-Item -ItemType Directory -Force -Path $installLoc | Out-Null

    Write-Info "Importing as WSL2 instance '$name'..."
    & wsl --import $name $installLoc $tempRootfs --version 2 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Err "wsl --import failed for $name"
        Remove-Item $tempRootfs -Force -ErrorAction SilentlyContinue
        exit 1
    }
    Remove-Item $tempRootfs -Force -ErrorAction SilentlyContinue

    Persist-CelestiaInstanceEnv -Name $name
    Write-Ok "Isolated WSL2 instance created: $name"
    Write-Ok "  Base:     Alpine Linux (~3 MB rootfs)"
    Write-Ok "  Location: $installLoc"
    Write-Ok "  Env var:  CELESTIA_WSL_INSTANCE=$name (persisted to User scope)"
    return $name
}

# ── Phase 1: Prerequisites (Windows) ────────────────────────────────────────

function Test-WindowsRust {
    Write-Step "Phase 1: Checking Windows prerequisites"
    try {
        $ver = & rustc --version 2>&1
        if ($LASTEXITCODE -eq 0) { Write-Ok "Windows Rust: $ver"; return $true }
    } catch { }
    Write-Warn "Windows rustc not found — scriptum/shittim-chest builds will fail."
    Write-Warn "Install from https://rustup.rs then re-run."
    return $false
}

function Ensure-WslAvailable {
    Write-Info "Checking WSL2 availability..."
    $wslOk = $false
    try { $null = & wsl --status 2>&1; $wslOk = ($LASTEXITCODE -eq 0) } catch { }
    if (-not $wslOk) {
        Write-Err "WSL2 is not available. Install with: wsl --install"
        Write-Err "After reboot, re-run this script."
        exit 1
    }
    Write-Ok "WSL2 is available"
}

function Resolve-SourceRoot {
    param([string]$Hint)
    if ($Hint -and (Test-Path (Join-Path $Hint "entelecheia\Cargo.toml"))) {
        return (Resolve-Path -LiteralPath $Hint).Path
    }
    # arona/scripts/install/ → ../../.. = celestia source root
    $try = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..") -ErrorAction SilentlyContinue).Path
    if ($try -and (Test-Path (Join-Path $try "entelecheia\Cargo.toml"))) { return $try }
    if (-not $Quick) {
        $input = Read-Host "Enter celestia source root path"
        if ($input -and (Test-Path (Join-Path $input "entelecheia\Cargo.toml"))) {
            return (Resolve-Path -LiteralPath $input).Path
        }
    }
    Write-Err "Could not resolve celestia source root (must contain entelecheia\Cargo.toml)."
    Write-Err "Re-run with -SourceRoot <path>."
    exit 1
}

# ── Phase 2: Initialize WSL2 instance ────────────────────────────────────────

function Initialize-WslInstance {
    Write-Step "Phase 2: Initializing WSL2 instance ($($script:WSLDistro))"

    # Run celestia-init.sh inside the WSL2 instance.
    # The script is accessed via the /mnt/ path (auto-mounted in all WSL2 instances).
    $initScript = Join-Path $PSScriptRoot "celestia-init.sh"
    if (Test-Path $initScript) {
        $wslInitPath = Convert-WinPathToWSL -Path $initScript

        # Extract numeric instance ID from celestia-NNN
        $instanceId = 0
        if ($script:WSLDistro -match 'celestia-(\d{3})$') {
            $instanceId = [int]$Matches[1]
        }

        Write-Info "Running celestia-init.sh inside $($script:WSLDistro) (id=$instanceId)..."
        # Use sh (Alpine default), not bash (may not be installed yet).
        # Pass CELESTIA_INSTANCE_ID so the init script uses the correct port offset.
        $r = Invoke-WSL -Command "CELESTIA_INSTANCE_ID=$instanceId sh '$wslInitPath' 2>&1" -NoProfile
        if ($r) {
            $r | ForEach-Object {
                if     ($_ -match '\[INIT\].*(?:MISSING|FAILED)|error')   { Write-Host $_ -ForegroundColor Red }
                elseif ($_ -match '\[INIT\].*OK|\[INIT\].*complete')      { Write-Host $_ -ForegroundColor Green }
                elseif ($_ -match '\[INIT\].*==>')                        { Write-Host $_ -ForegroundColor Cyan }
                elseif ($_ -match '\[INIT\].*WARN')                       { Write-Host $_ -ForegroundColor Yellow }
                else { Write-Host $_ }
            }
        }
        # Verify podman is running after init.
        if (-not (Invoke-WSL -Command "podman info &>/dev/null && echo running" -NoProfile)) {
            Write-Warn "Podman may not be running. Start manually: wsl -d $($script:WSLDistro) -u root podman system service --time=0 unix:///run/podman/podman.sock &"
        } else {
            Write-Ok "Podman is running inside $($script:WSLDistro)"
        }
    } else {
        Write-Warn "celestia-init.sh not found at $initScript — skipping WSL initialization."
    }
}

# ── Phase 3: Build entelecheia + evernight in WSL2 ──────────────────────────

function Set-ProjectSymlinkInWSL {
    param([string]$WslSourcePath)
    Write-Step "Phase 3: Linking source into WSL workspace"
    $r = Invoke-WSL -Command @"
set -euo pipefail
SRC="$WslSourcePath"
LINK="`$HOME/projects/celestia"
mkdir -p "`$(dirname "`$LINK")"
if [[ -L "`$LINK" ]]; then rm -f "`$LINK"; fi
if [[ ! -e "`$LINK" ]]; then ln -s "`$SRC" "`$LINK"; echo "LINKED"; else echo "EXISTS"; fi
"@ -NoProfile
    if ($r -match "LINKED") { Write-Ok "Symlink created: ~/projects/celestia -> $WslSourcePath" }
    else                    { Write-Ok "Symlink already exists: ~/projects/celestia" }
}

function Build-EntelecheiaInWSL {
    param([string]$WslSourcePath)
    if ($SkipBuild) { Write-Info "Skipping entelecheia build (-SkipBuild)"; return $false }
    Write-Step "Building entelecheia scepter (release) in WSL"
    $dir = "$WslSourcePath/entelecheia"
    if (-not (Test-WSLCommand "rustc")) {
        Write-Info "Installing Rust in WSL via rustup..."
        Invoke-WSL -Command "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y" -NoProfile | Out-Null
    }
    $r = Invoke-WSL -Command @"
set -euo pipefail
source "`$HOME/.cargo/env" 2>/dev/null || true
cd "$dir"
cargo build --release -p scepter 2>&1 && echo BUILD_OK || echo BUILD_FAILED
"@ -NoProfile
    if ($r -match "BUILD_OK") { Write-Ok "scepter built (release)"; return $true }
    Write-Err "scepter build failed. Run: wsl bash -lc 'cd $dir && cargo build --release -p scepter'"
    return $false
}

function Build-EvernightInWSL {
    param([string]$WslSourcePath)
    if ($SkipBuild) { Write-Info "Skipping evernight build (-SkipBuild)"; return $false }
    Write-Step "Building evernight (release) in WSL"
    $dir = "$WslSourcePath/evernight"
    $r = Invoke-WSL -Command @"
set -euo pipefail
source "`$HOME/.cargo/env" 2>/dev/null || true
cd "$dir"
cargo build --release -p evernight 2>&1 && echo BUILD_OK || echo BUILD_FAILED
"@ -NoProfile
    if ($r -match "BUILD_OK") { Write-Ok "evernight built (release)"; return $true }
    Write-Err "evernight build failed. Run: wsl bash -lc 'cd $dir && cargo build --release -p evernight'"
    return $false
}

function Set-EntelecheiaEnv {
    param([string]$WslSourcePath)
    Write-Step "Ensuring entelecheia/.env exists"
    $r = Invoke-WSL -Command @"
set -euo pipefail
cd "$WslSourcePath/entelecheia"
if [[ -f .env ]]; then echo "ENV_EXISTS"
elif [[ -f .env.example.minimal ]]; then cp .env.example.minimal .env; echo "ENV_FROM_MINIMAL"
elif [[ -f .env.example ]]; then cp .env.example .env; echo "ENV_FROM_EXAMPLE"
else
    cat > .env <<'ENVEOF'
LLM_API_KEY=sk-your-key-here
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-4o
DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
SERVER_BIND_ADDRESS=127.0.0.1:8424
RUST_LOG=info
ENVEOF
    echo "ENV_MINIMAL_CREATED"
fi
"@ -NoProfile
    if     ($r -match "ENV_EXISTS")         { Write-Ok ".env already exists" }
    elseif ($r -match "ENV_FROM_MINIMAL")   { Write-Ok ".env created from .env.example.minimal" }
    elseif ($r -match "ENV_FROM_EXAMPLE")   { Write-Ok ".env created from .env.example" }
    elseif ($r -match "ENV_MINIMAL_CREATED"){ Write-Ok ".env created (minimal)" }
    Write-Warn "Edit LLM_API_KEY etc.:  wsl -d $($script:WSLDistro) nano $WslSourcePath/entelecheia/.env"
}

function Start-PostgresInWSL {
    param([string]$WslSourcePath)
    Write-Step "Starting PostgreSQL via docker compose"
    $dir = "$WslSourcePath/entelecheia"
    $r = Invoke-WSL -Command @"
set -euo pipefail
cd "$dir"
if [[ -f tests/docker/docker-compose.e2e.yml ]]; then COMPOSE_FILE="tests/docker/docker-compose.e2e.yml"
elif [[ -f docker-compose.yml ]]; then COMPOSE_FILE="docker-compose.yml"
else echo "NO_COMPOSE"; exit 0; fi
docker compose -f "`$COMPOSE_FILE" up -d postgres 2>&1 || \
    docker-compose -f "`$COMPOSE_FILE" up -d postgres 2>&1 || { echo "COMPOSE_FAILED"; exit 1; }
echo "Waiting for PostgreSQL..."
for i in `$(seq 1 30); do
    if docker ps --format '{{.Names}} {{.Status}}' | grep -i postgres | grep -qi healthy; then echo "PG_READY"; exit 0; fi
    if docker exec "`$(docker ps --filter name=postgres --format '{{.Names}}' | head -1)" pg_isready -U amphoreus 2>/dev/null || \
       docker exec "`$(docker ps --filter name=postgres --format '{{.Names}}' | head -1)" pg_isready -U entelecheia 2>/dev/null; then
        echo "PG_READY"; exit 0
    fi
    sleep 2
done
echo "PG_NOT_READY"
"@ -NoProfile
    if     ($r -match "NO_COMPOSE")    { Write-Warn "No docker-compose file found — skip postgres start" }
    elseif ($r -match "PG_READY")      { Write-Ok "PostgreSQL is ready" }
    elseif ($r -match "PG_NOT_READY")  { Write-Warn "PostgreSQL did not become ready in 60s" }
    elseif ($r -match "COMPOSE_FAILED"){ Write-Err "docker compose up failed" }
    else                               { Write-Warn "Postgres start result: $r" }
}

# ── Phase 4: Build scriptum + shittim-chest on Windows ───────────────────────

function Invoke-CargoBuild {
    param([string]$Dir, [string[]]$CargoArgs)
    Push-Location $Dir
    try {
        & cargo @CargoArgs 2>&1 | ForEach-Object {
            if ($_ -is [string]) { Write-Host $_ } else { Write-Host $_.ToString() }
        }
        return ($LASTEXITCODE -eq 0)
    } finally { Pop-Location }
}

function Build-ScriptumOnWindows {
    param([string]$SourceRoot)
    if ($SkipBuild) { Write-Info "Skipping scriptum build (-SkipBuild)"; return $false }
    Write-Step "Phase 4: Building scriptum (TUI) on Windows"
    $dir = Join-Path $SourceRoot "scriptum"
    if (-not (Test-Path (Join-Path $dir "Cargo.toml"))) {
        Write-Warn "scriptum/ not found at $dir — skipping"; return $false
    }
    if (Invoke-CargoBuild -Dir $dir -Args @("build","--release","--bin","scriptum")) {
        $exe = Join-Path $dir "target\release\scriptum.exe"
        if (Test-Path $exe) { Write-Ok "scriptum built: $exe"; return $true }
    }
    Write-Err "scriptum build failed"
    return $false
}

function Build-ShittimChestOnWindows {
    param([string]$SourceRoot, [ref]$OutBinary)
    if ($SkipBuild) { Write-Info "Skipping shittim-chest build (-SkipBuild)"; return $false }
    Write-Step "Building shittim-chest on Windows"
    $dir = Join-Path $SourceRoot "shittim-chest"
    if (-not (Test-Path (Join-Path $dir "Cargo.toml"))) {
        Write-Warn "shittim-chest/ not found at $dir — skipping"; return $false
    }
    Push-Location $dir
    try {
        # Try Tauri first (GUI app). `cargo tauri build` needs cargo-tauri.
        $tauriCli = $false
        try { $null = & cargo tauri --version 2>&1; if ($LASTEXITCODE -eq 0) { $tauriCli = $true } } catch { }

        if ($tauriCli) {
            Write-Info "Attempting Tauri build (may take several minutes)..."
            & cargo tauri build 2>&1 | ForEach-Object { if ($_ -is [string]) { Write-Host $_ } else { Write-Host $_.ToString() } }
            if ($LASTEXITCODE -eq 0) {
                $bundleExe = Get-ChildItem -Path "target\release\bundle" -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue |
                    Select-Object -First 1
                if ($bundleExe) {
                    $OutBinary.Value = $bundleExe.FullName
                    Write-Ok "Tauri app built: $($bundleExe.FullName)"; return $true
                }
            }
            Write-Warn "Tauri build did not produce a bundle. Falling back to web UI / CLI."
        } else {
            Write-Info "cargo-tauri not installed. Trying web UI + CLI build."
        }

        # Build the web UI (pnpm --filter @celestia-island/webui build)
        if (Get-Command pnpm -ErrorAction SilentlyContinue) {
            Write-Info "Building web UI..."
            & pnpm --filter @celestia-island/webui build 2>&1 | ForEach-Object { if ($_ -is [string]) { Write-Host $_ } else { Write-Host $_.ToString() } }
            if ($LASTEXITCODE -ne 0) { Write-Warn "Web UI build returned non-zero." }
        } else { Write-Warn "pnpm not found — skipping web UI build." }

        # Build the CLI binary (default member of the workspace).
        Write-Info "Building shittim-chest CLI (chest-cli)..."
        & cargo build --release -p shittim-chest-cli 2>&1 | ForEach-Object { if ($_ -is [string]) { Write-Host $_ } else { Write-Host $_.ToString() } }
        if ($LASTEXITCODE -eq 0) {
            $exe = Join-Path $dir "target\release\chest-cli.exe"
            if (Test-Path $exe) { $OutBinary.Value = $exe; Write-Ok "shittim-chest CLI built: $exe"; return $true }
        }
        Write-Err "shittim-chest build failed."; return $false
    } finally { Pop-Location }
}

# ── Phase 5: Start Menu shortcuts ───────────────────────────────────────────

function New-Shortcut {
    param([string]$ShortcutPath, [string]$TargetPath, [string]$Arguments = "",
          [string]$WorkingDirectory = "", [string]$Description = "")
    $shell = New-Object -ComObject WScript.Shell
    $sc = $shell.CreateShortcut($ShortcutPath)
    $sc.TargetPath = $TargetPath
    if ($Arguments)        { $sc.Arguments = $Arguments }
    if ($WorkingDirectory) { $sc.WorkingDirectory = $WorkingDirectory }
    if ($Description)      { $sc.Description = $Description }
    $sc.Save()
}

function New-TerminalShortcut {
    param([string]$ShortcutPath, [string]$ExePath, [string]$Description)
    # Use Windows Terminal (wt.exe) if available; otherwise plain powershell host.
    $wt = Get-Command wt.exe -ErrorAction SilentlyContinue
    if ($wt) {
        New-Shortcut $ShortcutPath $wt.Source `
            "-p `"Windows PowerShell`" -c `"$ExePath`"" $script:InstallDir $Description
    } else {
        $psExe = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
        New-Shortcut $ShortcutPath $psExe "-NoExit -Command `"$ExePath`"" $script:InstallDir $Description
    }
}

function Install-Shortcuts {
    param([string]$SourceRoot, [string]$ScriptumExe, [string]$ShittimExe)
    if ($SkipShortcuts) { Write-Info "Skipping Start Menu shortcuts (-SkipShortcuts)"; return }
    Write-Step "Phase 5: Installing Start Menu shortcuts"
    New-Item -ItemType Directory -Force -Path $script:InstallDir   | Out-Null
    New-Item -ItemType Directory -Force -Path $script:StartMenuDir | Out-Null

    if ($ScriptumExe -and (Test-Path $ScriptumExe)) {
        $dest = Join-Path $script:InstallDir "scriptum.exe"
        Copy-Item -LiteralPath $ScriptumExe -Destination $dest -Force
        New-TerminalShortcut (Join-Path $script:StartMenuDir "Scriptum.lnk") $dest "Celestia Scriptum TUI"
        Write-Ok "Scriptum shortcut created"
    } else { Write-Warn "scriptum.exe not available — no shortcut created." }

    if ($ShittimExe -and (Test-Path $ShittimExe)) {
        $destName = Split-Path $ShittimExe -Leaf
        $dest = Join-Path $script:InstallDir $destName
        Copy-Item -LiteralPath $ShittimExe -Destination $dest -Force
        if ($destName -notmatch "chest-cli") {
            # Tauri GUI .exe — launch directly
            New-Shortcut (Join-Path $script:StartMenuDir "Shittim Chest.lnk") $dest `
                "" $script:InstallDir "Celestia Shittim-chest desktop app"
        } else {
            # CLI launcher — open a terminal so user sees logs
            New-TerminalShortcut (Join-Path $script:StartMenuDir "Shittim Chest.lnk") $dest "Celestia Shittim-chest (CLI)"
        }
        Write-Ok "Shittim-chest shortcut created"
    } else { Write-Warn "shittim-chest binary not available — no shortcut created." }
}

# ── Phase 6: Start services + summary ──────────────────────────────────────

function Start-ScepterInWSL {
    param([string]$WslSourcePath, [bool]$BuildOk)
    Write-Step "Phase 6: Starting scepter server in WSL"
    if (-not $BuildOk) {
        Write-Warn "scepter was not built — skipping auto-start."
        Write-Warn "Start manually: wsl -d $($script:WSLDistro) bash -lc 'cd $WslSourcePath/entelecheia && ./target/release/scepter'"
        return
    }
    $r = Invoke-WSL -Command @"
set -euo pipefail
source "`$HOME/.cargo/env" 2>/dev/null || true
cd "$WslSourcePath/entelecheia"
mkdir -p "`$HOME/.local/share/celestia/logs"
LOG="`$HOME/.local/share/celestia/logs/scepter.log"
PID_FILE="`$HOME/.local/share/celestia/logs/scepter.pid"
if [[ -f "`$PID_FILE" ]] && kill -0 "`$(cat "`$PID_FILE")" 2>/dev/null; then
    echo "ALREADY_RUNNING: `$(cat "`$PID_FILE")"; exit 0
fi
nohup ./target/release/scepter >"`$LOG" 2>&1 &
echo `$! > "`$PID_FILE"; sleep 2
if kill -0 "`$(cat "`$PID_FILE")" 2>/dev/null; then echo "STARTED: `$(cat "`$PID_FILE")"
else echo "FAILED"; fi
"@ -NoProfile
    if     ($r -match "ALREADY_RUNNING"){ Write-Ok "scepter already running (pid: $($r -replace '.*ALREADY_RUNNING:\s*',''))" }
    elseif ($r -match "STARTED")        { Write-Ok "scepter started (pid: $($r -replace '.*STARTED:\s*',''))" }
    elseif ($r -match "FAILED")         { Write-Err "scepter failed — check ~/.local/share/celestia/logs/scepter.log" }
    else                                { Write-Warn "scepter start result: $r" }
}

function Show-Summary {
    param([string]$WslSourcePath, [hashtable]$Built)
    $sep = "─" * 64
    Write-Host ""; Write-Host $sep
    Write-Host "  Celestia Installation Complete" -ForegroundColor Green
    Write-Host $sep; Write-Host ""
    Write-Host "  Components built:" -ForegroundColor Yellow
    foreach ($k in @(@("scepter (entelecheia)",  $Built.Scepter),
                     @("evernight",              $Built.Evernight),
                     @("scriptum (TUI)",         $Built.Scriptum),
                     @("shittim-chest",         $Built.Shittim))) {
        $name = $k[0].PadRight(22)
        $ok   = [bool]$k[1]
        Write-Host "    $name " -NoNewline
        Write-Host ($(if ($ok) {"[OK]"} else {"[MISSING]"})) -ForegroundColor $(if ($ok) {'Green'} else {'Red'})
    }
    Write-Host ""
    Write-Host "  Paths:" -ForegroundColor Yellow
    Write-Host "    WSL source:        $WslSourcePath"
    Write-Host "    WSL symlink:       ~/projects/celestia"
    Write-Host "    entelecheia/.env:  $WslSourcePath/entelecheia/.env"
    Write-Host "    scepter log:       ~/.local/share/celestia/logs/scepter.log"
    Write-Host "    Windows install:   $script:InstallDir"
    Write-Host "    Start Menu group:  $script:StartMenuDir"
    Write-Host ""
    Write-Host "  Services:" -ForegroundColor Yellow
    Write-Host "    scepter (HTTP/WS): http://localhost:$($script:ScepterPort)"
    Write-Host "    PostgreSQL:        localhost:5432 (in WSL2)"
    Write-Host ""
    Write-Host "  How to use:" -ForegroundColor Yellow
    Write-Host "    • Launch 'Scriptum' from the Start Menu → TUI connects to scepter."
    Write-Host "    • Launch 'Shittim Chest' from the Start Menu → desktop app / CLI."
    Write-Host "    • Edit .env:        wsl -d $($script:WSLDistro) nano $WslSourcePath/entelecheia/.env"
    Write-Host "    • Tail scepter log: wsl -d $($script:WSLDistro) tail -f ~/.local/share/celestia/logs/scepter.log"
    Write-Host ""
    Write-Host "  Stop / restart:" -ForegroundColor Yellow
    Write-Host "    Stop scepter:  wsl -d $($script:WSLDistro) bash -lc 'kill `$(cat ~/.local/share/celestia/logs/scepter.pid)'"
    Write-Host "    Stop postgres: wsl -d $($script:WSLDistro) bash -lc 'cd $WslSourcePath/entelecheia && docker compose -f tests/docker/docker-compose.e2e.yml down'"
    Write-Host "    Restart all:   re-run this script (idempotent)."
    Write-Host ""; Write-Host $sep
}

# ── Main ────────────────────────────────────────────────────────────────────

function Main {
    Write-Host ""; Write-Host ("─" * 64)
    Write-Host "  Celestia Unified Installer (Windows + WSL2)" -ForegroundColor Cyan
    Write-Host ("─" * 64)
    Write-Info "PowerShell $($PSVersionTable.PSVersion)"

    # Phase 1
    $hasWindowsRust = Test-WindowsRust
    Ensure-WslAvailable
    $script:WSLDistro = Resolve-CelestiaWslInstance
    Write-Ok "WSL instance: $script:WSLDistro (isolated, ignores Ubuntu-24.04)"


    $sourceRoot = Resolve-SourceRoot -Hint $SourceRoot
    Write-Ok "Celestia source root: $sourceRoot"

    # Phase 2
    if (-not $SkipDocker) {
        Initialize-WslInstance
    } else { Write-Info "Skipping WSL instance initialization (-SkipDocker)" }

    # Phase 3
    $wslSource = Convert-WinPathToWSL -Path $sourceRoot
    Write-Info "WSL view of source: $wslSource"
    Set-ProjectSymlinkInWSL -WslSourcePath $wslSource
    $scepterBuilt   = Build-EntelecheiaInWSL -WslSourcePath $wslSource
    $evernightBuilt = Build-EvernightInWSL  -WslSourcePath $wslSource
    Set-EntelecheiaEnv    -WslSourcePath $wslSource
    Start-PostgresInWSL   -WslSourcePath $wslSource

    # Phase 4
    $scriptumBuilt = $false; $shittimBuilt = $false; $shittimExe = ""
    if ($hasWindowsRust) {
        $scriptumBuilt = Build-ScriptumOnWindows -SourceRoot $sourceRoot
        $shittimBuilt  = Build-ShittimChestOnWindows -SourceRoot $sourceRoot -OutBinary ([ref]$shittimExe)
    } else { Write-Warn "Skipping Windows builds — Rust toolchain missing." }
    $scriptumExe = if ($scriptumBuilt) { Join-Path $sourceRoot "scriptum\target\release\scriptum.exe" } else { "" }

    # Phase 5
    Install-Shortcuts -SourceRoot $sourceRoot -ScriptumExe $scriptumExe -ShittimExe $shittimExe

    # Phase 6
    Start-ScepterInWSL -WslSourcePath $wslSource -BuildOk $scepterBuilt
    $stateJson = @{ Stage = "complete"; Time = (Get-Date).ToString("o") } | ConvertTo-Json -Compress
    [System.IO.File]::WriteAllText($script:StateFile, $stateJson, [System.Text.UTF8Encoding]::new($false))
    Show-Summary -WslSourcePath $wslSource -Built @{
        Scepter=$scepterBuilt; Evernight=$evernightBuilt; Scriptum=$scriptumBuilt; Shittim=$shittimBuilt
    }
}

# Reboot-resume support: clear stale state from a previous WSL install reboot.
if ((Test-Path $script:StateFile) -and ((Get-Content $script:StateFile -Raw | ConvertFrom-Json).Stage -eq "wsl-reboot-pending")) {
    Write-Info "Resuming after reboot..."
    Remove-Item $script:StateFile -Force -ErrorAction SilentlyContinue
}

try {
    Main
} catch {
    Write-Err "Installer failed: $($_.Exception.Message)"
    Write-Err "Stack: $($_.ScriptStackTrace)"
    exit 1
}
