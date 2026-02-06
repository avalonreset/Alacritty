param(
  [string]$Output = "THIRD_PARTY_NOTICES.html",
  [switch]$Offline
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Push-Location (Join-Path $PSScriptRoot "..")
try {
  $args = @("about", "generate", "--workspace", "--locked")
  if ($Offline) { $args += "--offline" }
  $args += @("about.hbs", "-o", $Output)

  # Requires `cargo-about` to be installed: `cargo install --locked cargo-about`
  cargo @args
} finally {
  Pop-Location
}

