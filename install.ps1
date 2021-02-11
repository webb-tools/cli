#!/usr/bin/env pwsh
# adopted from https://github.com/denoland/deno_install/blob/master/install.ps1

$ErrorActionPreference = 'Stop'

if ($v) {
  $Version = "v${v}"
}

if ($args.Length -eq 1) {
  $Version = $args.Get(0)
}

$CliInstall = "$Home\.webb"
$BinDir = $CliInstall
$CliZip = "$BinDir\webb.zip"
$CliExe = "$BinDir\webb.exe"
$Target = 'x86_64-pc-windows-msvc'

# GitHub requires TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$CliUri = if (!$Version) {
  "https://github.com/webb-tools/cli/releases/latest/download/webb-${Target}.zip"
} else {
  "https://github.com/webb-tools/cli/releases/download/${Version}/webb-${Target}.zip"
}

if (!(Test-Path $BinDir)) {
  New-Item $BinDir -ItemType Directory | Out-Null
}

Invoke-WebRequest $CliUri -OutFile $CliZip -UseBasicParsing

if (Get-Command Expand-Archive -ErrorAction SilentlyContinue) {
  Expand-Archive $CliZip -Destination $BinDir -Force
} else {
  if (Test-Path $CliExe) {
    Remove-Item $CliExe
  }
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  [IO.Compression.ZipFile]::ExtractToDirectory($CliZip, $BinDir)
}

Remove-Item $CliZip

$User = [EnvironmentVariableTarget]::User
$Path = [Environment]::GetEnvironmentVariable('Path', $User)
if (!(";$Path;".ToLower() -like "*;$BinDir;*".ToLower())) {
  [Environment]::SetEnvironmentVariable('Path', "$Path;$BinDir", $User)
  $Env:Path += ";$BinDir"
}

Write-Output "Webb CLI was installed successfully to $CliExe"
Write-Output "Run 'webb --help' to get started"
