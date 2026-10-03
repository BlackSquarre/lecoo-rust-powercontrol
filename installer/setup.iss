#ifndef AppVersion
  #error AppVersion must be supplied by the release script
#endif

[Setup]
AppId={{1877EAC4-62C1-4628-A32E-6BA630BE126A}
AppName=Lecoo Rust PowerControl
AppVersion={#AppVersion}
AppPublisher=BlackSquarre
AppPublisherURL=https://github.com/BlackSquarre/lecoo-rust-powercontrol
DefaultDirName={autopf}\Lecoo Rust PowerControl
DefaultGroupName=Lecoo Rust PowerControl
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
PrivilegesRequired=admin
OutputBaseFilename=lecoo-rust-powercontrol-v{#AppVersion}-windows-x64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
UninstallDisplayIcon={app}\lecoo-control-center.exe

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\lecoo-control-center.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\hardware_test.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "uninstall-startup.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Lecoo Rust PowerControl"; Filename: "{app}\lecoo-control-center.exe"
Name: "{autodesktop}\Lecoo Rust PowerControl"; Filename: "{app}\lecoo-control-center.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\lecoo-control-center.exe"; Description: "{cm:LaunchProgram,Lecoo Rust PowerControl}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File ""{app}\uninstall-startup.ps1"" -Executable ""{app}\lecoo-control-center.exe"""; Flags: runhidden; RunOnceId: "RemoveInstalledStartupTasks"
