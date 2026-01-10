; Urocissa Installer Script
; Requires NSIS 3.0+

;--------------------------------
;General

  ;Name and file
  Name "Urocissa"
  OutFile "urocissa-install-1.0.exe"
  Unicode True

  ;Default installation folder
  InstallDir "$PROGRAMFILES64\Urocissa"
  
  ;Get installation folder from registry if available
  InstallDirRegKey HKCU "Software\Urocissa" ""

  ;Request application privileges for Windows Vista/7/8/10
  RequestExecutionLevel admin

;--------------------------------
;Interface Settings

  !include "MUI2.nsh"

  !define MUI_ABORTWARNING
  !ifdef PRODUCT_ICON
    !define MUI_ICON "${PRODUCT_ICON}"
    !define MUI_UNICON "${PRODUCT_ICON}"
  !else
    !define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
    !define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
  !endif

;--------------------------------
;Pages

  !insertmacro MUI_PAGE_WELCOME
  !insertmacro MUI_PAGE_LICENSE "..\LICENSE"
  !insertmacro MUI_PAGE_DIRECTORY
  !insertmacro MUI_PAGE_INSTFILES
  !insertmacro MUI_PAGE_FINISH

  !insertmacro MUI_UNPAGE_WELCOME
  !insertmacro MUI_UNPAGE_CONFIRM
  !insertmacro MUI_UNPAGE_INSTFILES
  !insertmacro MUI_UNPAGE_FINISH

;--------------------------------
;Languages
 
  !insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Sections

Section "Urocissa Core" SecCore

  SetOutPath "$INSTDIR"
  
  ; Copy backend executable
  File "target\static-release\urocissa.exe"
  
  ; Copy Urocissa License
  File "..\LICENSE"
  
  ; Copy FFmpeg binaries
  SetOutPath "$INSTDIR\bin"
  File "bin\ffmpeg.exe"
  File "bin\ffprobe.exe"
  
  ; Copy FFmpeg License and Credits
  File "bin\FFMPEG_LICENSE.txt"
  File "bin\FFMPEG_README.txt"

  SetOutPath "$INSTDIR"
  
  ; Copy uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Store installation folder

  WriteRegStr HKCU "Software\Urocissa" "" $INSTDIR

  ; Create Start Menu Shortcuts
  CreateDirectory "$SMPROGRAMS\Urocissa"
  CreateShortcut "$SMPROGRAMS\Urocissa\Urocissa.lnk" "$INSTDIR\urocissa.exe" "" "$INSTDIR\urocissa.exe" 0
  CreateShortcut "$SMPROGRAMS\Urocissa\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

  ; Create Desktop Shortcut
  CreateShortcut "$DESKTOP\Urocissa.lnk" "$INSTDIR\urocissa.exe" "" "$INSTDIR\urocissa.exe" 0

  ; Pin to Taskbar (Windows 10/11)
  ; Note: This requires the shortcut to exist first
  DetailPrint "Creating taskbar shortcut..."
  CreateShortcut "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Urocissa.lnk" "$INSTDIR\urocissa.exe" "" "$INSTDIR\urocissa.exe" 0

SectionEnd

;--------------------------------
;Uninstaller Section

Section "Uninstall"

  ; Remove files
  Delete "$INSTDIR\urocissa.exe"
  Delete "$INSTDIR\config.json"
  Delete "$INSTDIR\Uninstall.exe"
  
  ; Remove assets
  RMDir /r "$INSTDIR\assets"
  RMDir /r "$INSTDIR\bin" ; Remove auto-downloaded ffmpeg binaries
  
  ; Note: We intentionally DO NOT delete the User Data (AppData) here
  ; to preserve user photos/database upon uninstall.
  
  ; Remove shortcuts
  Delete "$SMPROGRAMS\Urocissa\Urocissa.lnk"
  Delete "$SMPROGRAMS\Urocissa\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Urocissa"

  ; Remove desktop shortcut
  Delete "$DESKTOP\Urocissa.lnk"

  ; Remove taskbar shortcut
  Delete "$APPDATA\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar\Urocissa.lnk"

  ; Remove registry keys
  DeleteRegKey /ifempty HKCU "Software\Urocissa"

  ; Remove install directory (only if empty)
  RMDir "$INSTDIR"

SectionEnd
