; Inno Setup 7 Script for BMI CXR-AI v2 (Rust Clinical Workstation)
; Compatible with Inno Setup 6 & 7

#define MyAppName "BMI CXR-AI v2"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "BMI Research Team"
#define MyAppURL "https://github.com/"
#define MyAppExeName "cxr_medic_rust.exe"
#define MyAppIcoName "app_icon.ico"

[Setup]
; App Identity
AppId={{E79B5A43-1D3B-4DF9-99E8-74B6429A26A1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}

; Destination Directories
DefaultDirName={autopf}\BMI CXR-AI v2
DefaultGroupName={#MyAppName}
AllowNoIcons=yes

; Output Configuration
OutputDir=Output
OutputBaseFilename=BMI_CXR_AI_v2_Setup_v{#MyAppVersion}
SetupIconFile=..\assets\icons\{#MyAppIcoName}
UninstallDisplayIcon={app}\{#MyAppIcoName}

; Compression & Modern Wizard Styling
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
WizardSizePercent=100

; System Architecture
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0

; Privileges & Control Panel Entry
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
DisableProgramGroupPage=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main Standalone Rust Executable
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

; Application Icon for shortcuts
Source: "..\assets\icons\{#MyAppIcoName}"; DestDir: "{app}"; Flags: ignoreversion

; Documentation
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
; Start Menu Shortcut
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppIcoName}"

; Start Menu Uninstaller
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"

; Desktop Shortcut (Optional Task)
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppIcoName}"; Tasks: desktopicon

[Run]
; Option to launch application immediately after setup finishes
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

