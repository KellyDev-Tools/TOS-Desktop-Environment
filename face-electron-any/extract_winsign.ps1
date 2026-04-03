$CacheDir = "$env:LOCALAPPDATA\electron-builder\Cache\winCodeSign"
$TargetDir = "$CacheDir\winCodeSign-2.6.0"
$Archive = "$CacheDir\winCodeSign.7z"

if (-Not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Force -Path $CacheDir | Out-Null
    Invoke-WebRequest -Uri "https://github.com/electron-userland/electron-builder-binaries/releases/download/winCodeSign-2.6.0/winCodeSign-2.6.0.7z" -OutFile $Archive
    
    # We expect this to exit with '2' on Windows due to darwin symlink errors, but the important Windows files will be there
    & "z:\repos\TOS-Desktop-Environment\face-electron-any\node_modules\7zip-bin\win\x64\7za.exe" x -bd $Archive "-o$TargetDir"
    
    Remove-Item $Archive -Force
    Write-Host "Done pre-extracting winCodeSign"
} else {
    Write-Host "winCodeSign already extracted!"
}
