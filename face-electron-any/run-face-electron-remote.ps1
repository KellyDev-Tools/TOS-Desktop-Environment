param (
    [string]$BrainHost = "192.168.68.77",
    [int]$BrainPort = 7001,
    [switch]$DevMode = $true
)

Write-Host "=============================================" -ForegroundColor Cyan
Write-Host " TOS Electron - Connect to Remote Brain" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "Target Host: $BrainHost"
Write-Host "Target Port: $BrainPort"
Write-Host "Mode:        $(If ($DevMode) { 'Development' } Else { 'Production' })"
Write-Host "=============================================" -ForegroundColor Cyan

# 1. Probe the remote Brain host to see if it is reachable
Write-Host "Probing remote Brain at ${BrainHost}:${BrainPort}..." -NoNewline
try {
    $tcp = New-Object System.Net.Sockets.TcpClient
    $connect = $tcp.BeginConnect($BrainHost, $BrainPort, $null, $null)
    $success = $connect.AsyncWaitHandle.WaitOne(2000, $false)
    if ($success) {
        $tcp.EndConnect($connect)
        Write-Host " [REACHABLE] ✅" -ForegroundColor Green
    } else {
        Write-Host " [UNREACHABLE] ❌" -ForegroundColor Yellow
        Write-Warning "The remote Brain at ${BrainHost}:${BrainPort} did not respond. Is it running on the Linux host?"
        Write-Warning "Proceeding anyway..."
    }
    $tcp.Close()
} catch {
    Write-Host " [ERROR] ❌" -ForegroundColor Red
    Write-Warning "Error probing remote host: $_"
    Write-Warning "Proceeding anyway..."
}

# 2. Check and build Svelte UI if needed
$SvelteBuild = Join-Path (Get-Item -Path ".." ).FullName "face-svelte-ui\build\index.html"
if (-not (Test-Path $SvelteBuild)) {
    Write-Host "Svelte UI build not found. Compiling Svelte UI..." -ForegroundColor Cyan
    Push-Location "..\face-svelte-ui"
    npm run build
    Pop-Location
}

# 3. Set environment variable for Electron
$env:TOS_BRAIN_WS = "wss://${BrainHost}:${BrainPort}"
Write-Host "TOS_BRAIN_WS env var set to: $env:TOS_BRAIN_WS" -ForegroundColor Gray

# 4. Start Electron app
if ($DevMode) {
    Write-Host "Launching Electron in Development Mode (--dev)..." -ForegroundColor Green
    npm run dev
} else {
    Write-Host "Launching Electron in Production Mode..." -ForegroundColor Green
    npm run start
}
