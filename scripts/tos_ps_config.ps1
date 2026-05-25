# PowerShell configuration analogous to tos_zsh_config.sh
# ----------------------------------------------------------
# Dot-sourcing check: we define the aliases as functions globally so they exist in the parent scope when loaded
# Let's make sure the functions are defined globally or in the caller scope.
# This script defines convenient Git aliases and sets a few PowerShell preferences.
# It can be dot-sourced (.\tots_ps_config.ps1) from your PowerShell profile
# to make the aliases available in every session.

# ----------------------------------------------------------
# PowerShell Preferences (similar spirit to Zsh options)
# ----------------------------------------------------------
# Prefer to stop on all errors (like set -e in shells)
$ErrorActionPreference = 'Stop'
# Do not store command history in the session (similar to HIST_NO_STORE)
$MaximumHistoryCount = 0
# Enable tab completion for commands (default in PowerShell)
if (Get-Command Set-PSReadLineOption -ErrorAction SilentlyContinue) { if ((Get-Command Set-PSReadLineOption).Parameters['PredictionSource']) { Set-PSReadLineOption -PredictionSource History } }

# ----------------------------------------------------------
# Git Aliases (implemented as PowerShell functions)
# ----------------------------------------------------------
# Remove built-in gcm alias (which maps to Get-Command) to prevent conflicts
if (Get-Alias gcm -ErrorAction SilentlyContinue) { Remove-Alias -Name gcm -Force -Scope Global -ErrorAction SilentlyContinue }

function global:ga { git add $args }
function global:gaa { git add --all $args }
function global:gap { git add --patch $args }
function global:gar { git apply --reject $args }
function global:gbv { git branch -vv $args }
function global:gca { git commit --amend -C HEAD $args }
function global:gclean { git clean -Xdf $args }
function global:gco { git checkout $args }
function global:gdh { git diff HEAD $args }
function global:gds { git diff --staged $args }
function global:gdu { git diff $args }
function global:gfa { git fetch --all $args }
function global:gp { git pull --stat $args }
function global:gpr { git pull --stat --rebase $args }
function global:gr { git remote -v $args }
function global:gra { git rebase --abort $args }
function global:grc { git rebase --continue $args }
function global:gri { git rebase --interactive $args }
function global:gs { git status --short $args }
function global:gsi { git status --short --ignored $args }
function global:gsl { git status $args }
function global:gsli { git status --ignored $args }
function global:gl { git log --oneline --decorate -n 10 $args }
# Dynamic Abbreviation Expansions with cursor placement (expands on Space)
if (Get-Command Set-PSReadLineKeyHandler -ErrorAction SilentlyContinue) {
    Set-PSReadLineKeyHandler -Chord ' ' -ScriptBlock {
        $line = $null
        $cursor = $null
        [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)
        $textBeforeCursor = $line.Substring(0, $cursor)
        $textAfterCursor = $line.Substring($cursor)
        if ($textBeforeCursor -match '\bgcm$') {
            $newLine = ($textBeforeCursor -replace '\bgcm$', 'git commit -m ""') + $textAfterCursor
            $newCursor = $cursor - 3 + 15
            [Microsoft.PowerShell.PSConsoleReadLine]::SetBufferState($newLine, $newCursor)
        } elseif ($textBeforeCursor -match '\bgcs$') {
            $newLine = ($textBeforeCursor -replace '\bgcs$', 'git  --compact-summary') + $textAfterCursor
            $newCursor = $cursor - 3 + 4
            [Microsoft.PowerShell.PSConsoleReadLine]::SetBufferState($newLine, $newCursor)
        } else {
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert(' ')
        }
    }
}
# Cleaned up old function signatures for gcm/gcs since they are replaced by dynamic expansions above.
function global:gcm {
    param([string]$msg)
    if (-not $msg) {
        $msg = Read-Host -Prompt "Commit message"
    }
    git commit -m $msg
}
function global:gcs {
    param([string]$opts)
    git $opts --compact-summary
}
function global:grh { git reset --hard $args }
function global:grm { git reset --mixed $args }
function global:grs { git reset --soft $args }

# ----------------------------------------------------------
# Option Emulations (similar spirit to Zsh options)
# ----------------------------------------------------------
# Emulate AUTO_CD: Typing a directory name directly will cd into it
$ExecutionContext.InvokeCommand.CommandNotFoundAction = {
    param($commandName, $commandLookupEventArgs)
    # Check if the command entered is actually a valid directory path
    $resolvedPath = Resolve-Path $commandName -ErrorAction SilentlyContinue
    if ($resolvedPath -and $resolvedPath.Provider.Name -eq "FileSystem" -and (Test-Path -Path $resolvedPath.Path -PathType Container)) {
        Set-Location $resolvedPath.Path
        # Tell PowerShell we handled the command execution
        $commandLookupEventArgs.CommandScriptBlock = { }
    }
}

# ----------------------------------------------------------
# Completions (kubectl, just, delta)
# ----------------------------------------------------------
if (Get-Command kubectl -ErrorAction SilentlyContinue) {
    try { kubectl completion powershell | Out-String | Invoke-Expression } catch {}
}
if (Get-Command just -ErrorAction SilentlyContinue) {
    try { just --completions powershell | Out-String | Invoke-Expression } catch {}
}
if (Get-Command delta -ErrorAction SilentlyContinue) {
    # delta has completions generator for powershell in newer versions
    try { delta --generate-completion powershell | Out-String | Invoke-Expression } catch {}
}

# End of tos_ps_config.ps1
