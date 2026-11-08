#Requires -RunAsAdministrator

$ErrorActionPreference = 'Stop'
$bootstrapper = Join-Path ([System.IO.Path]::GetTempPath()) (
    'vs_buildtools-' + [guid]::NewGuid().ToString('N') + '.exe'
)

try {
    Write-Host 'Загрузка установщика Visual Studio Build Tools 2022.'
    Invoke-WebRequest -UseBasicParsing -Uri 'https://aka.ms/vs/17/release/vs_buildtools.exe' -OutFile $bootstrapper

    Write-Host 'Установка средств MSVC и Windows SDK.'
    $installation = Start-Process -FilePath $bootstrapper -ArgumentList @(
        '--passive', '--wait', '--norestart',
        '--add', 'Microsoft.VisualStudio.Workload.VCTools', '--includeRecommended'
    ) -Wait -PassThru

    switch ($installation.ExitCode) {
        0 { Write-Host 'Установка завершена. Откройте новое окно PowerShell.' }
        3010 { Write-Host 'Установка завершена. Перезагрузите Windows перед сборкой.' }
        default { throw "Установщик завершился с кодом $($installation.ExitCode)." }
    }
} finally {
    if (Test-Path -LiteralPath $bootstrapper -PathType Leaf) {
        Remove-Item -LiteralPath $bootstrapper
    }
}
