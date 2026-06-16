; NSIS Installer for lrn - Markdown Note taking TUI
Unicode True

!include "MUI2.nsh"

Name "lrn"
OutFile "..\target\nsis-x86_64\release\lrn-installer.exe"
RequestExecutionLevel user
InstallDir "$LOCALAPPDATA\Programs\lrn"
SetCompressor /SOLID lzma

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install lrn"
  SetOutPath "$INSTDIR"
  File "..\target\x86_64-pc-windows-msvc\release\lrn.exe"
  
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
  ; Create Start Menu shortcut
  CreateDirectory "$SMPROGRAMS\lrn"
  CreateShortCut "$SMPROGRAMS\lrn\lrn.lnk" "$INSTDIR\lrn.exe" "" "" 0
  CreateShortCut "$SMPROGRAMS\lrn\Uninstall lrn.lnk" "$INSTDIR\uninstall.exe"
  
  ; Add to user PATH so "lrn" works from any terminal
  ReadEnvStr $R0 USERPATH
  IfFileExists "$R0\*"+3 +2
    StrCpy $R0 ""
  
  CreateDirectory "$LOCALAPPDATA\bin"
  WriteRegExpandStr HKCU "Environment" "PATH" "$LOCALAPPDATA\bin;$INSTDIR;%USERPATH%"
  
  ; Registry uninstall info
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\lrn" \
    "DisplayName" "lrn"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\lrn" \
    "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\lrn" \
    "DisplayIcon" "$INSTDIR\lrn.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\lrn" \
    "InstallLocation" "$INSTDIR"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\lrn\lrn.lnk"
  Delete "$SMPROGRAMS\lrn\Uninstall lrn.lnk"
  RmDir "$SMPROGRAMS\lrn"
  
  Delete "$INSTDIR\lrn.exe"
  Delete "$INSTDIR\uninstall.exe"
  RmDir "$INSTDIR"
  
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\lrn"
SectionEnd
