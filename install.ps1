$ErrorActionPreference = 'Stop'
$repo = 'Giafn/rusdi-rust'
$version = if ($env:RUDIS_LITE_VERSION) { $env:RUDIS_LITE_VERSION } else { 'latest' }
if (-not [Environment]::Is64BitOperatingSystem) { throw 'Unsupported platform: Windows x86 is not supported.' }
$asset = 'rudis-lite-windows-x86_64.exe'
$base = if ($version -eq 'latest') { "https://github.com/$repo/releases/latest/download" } else { "https://github.com/$repo/releases/download/$version" }
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("rudis-lite-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
  $binary = Join-Path $tmp $asset
  $sums = Join-Path $tmp 'SHA256SUMS'
  Invoke-WebRequest "$base/$asset" -OutFile $binary
  Invoke-WebRequest "$base/SHA256SUMS" -OutFile $sums
  $expected = ((Get-Content $sums | Where-Object { $_ -match "\s\*?$([regex]::Escape($asset))$" } | Select-Object -First 1) -split '\s+')[0]
  if (-not $expected) { throw "No checksum found for $asset" }
  $actual = (Get-FileHash $binary -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $expected.ToLowerInvariant()) { throw "Checksum mismatch for $asset" }
  $binDir = if ($env:RUDIS_LITE_BIN) { $env:RUDIS_LITE_BIN } else { Join-Path $HOME 'bin' }
  New-Item -ItemType Directory -Force -Path $binDir | Out-Null
  $destination = Join-Path $binDir 'rudis-lite.exe'
  Move-Item -Force $binary $destination
  Write-Host "Installed rudis-lite to $destination"
} finally { Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue }
