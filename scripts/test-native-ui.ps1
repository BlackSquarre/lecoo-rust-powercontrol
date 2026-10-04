[CmdletBinding()]
param([string]$Executable,[string]$ReportPath)
$ErrorActionPreference='Stop'
$taskRoot=Split-Path -Parent $PSScriptRoot
if(!$Executable){$Executable=Join-Path $taskRoot 'target\release\lecoo-control-center.exe'}
if(!$ReportPath){$ReportPath=Join-Path $taskRoot 'logs\native-migration\native-ui.json'}
New-Item -ItemType Directory -Path (Split-Path -Parent $ReportPath) -Force | Out-Null
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public class NativeUiTest {
 public delegate bool EnumProc(IntPtr h,IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f,IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h,EnumProc f,IntPtr p);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder s,int n);
 [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr GetProp(IntPtr h,string name);
 [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr h,int id);
 [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int c);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern uint GetGuiResources(IntPtr p,uint type);
 [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
 [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h,IntPtr dc);
 [DllImport("gdi32.dll")] public static extern uint GetPixel(IntPtr dc,int x,int y);
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h,int a,out int v,int size);
 [DllImport("user32.dll",EntryPoint="SendMessageW",CharSet=CharSet.Unicode)] public static extern IntPtr ReadText(IntPtr h,uint m,IntPtr w,StringBuilder text);
 public static string ControlText(IntPtr h){var s=new StringBuilder(4096);ReadText(h,0xD,(IntPtr)s.Capacity,s);return s.ToString();}
 public static string Text(IntPtr h){var s=new StringBuilder(2048);GetWindowText(h,s,s.Capacity);return s.ToString();}
 public static IntPtr[] Windows(int pid){var a=new List<IntPtr>();EnumWindows((h,p)=>{uint id;GetWindowThreadProcessId(h,out id);if(id==pid&&IsWindowVisible(h))a.Add(h);return true;},IntPtr.Zero);return a.ToArray();}
 public static IntPtr[] Children(IntPtr h){var a=new List<IntPtr>();EnumChildWindows(h,(c,p)=>{a.Add(c);return true;},IntPtr.Zero);return a.ToArray();}
 public static uint Pixel(IntPtr h){var dc=GetDC(h);try{return GetPixel(dc,5,5);}finally{ReleaseDC(h,dc);}}
}
'@
function Click([IntPtr]$Window,[int]$Id){$h=[NativeUiTest]::GetDlgItem($Window,$Id);if($h -eq [IntPtr]::Zero){throw "Missing control $Id"};$null=[NativeUiTest]::SendMessage($h,0xF5,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 600}
function MainWindow {return @([NativeUiTest]::Windows($script:taskProcess.Id) | Where-Object {[NativeUiTest]::Text($_) -like 'Lecoo Rust PowerControl*'}) | Select-Object -First 1}
function DialogWindow {return @([NativeUiTest]::Windows($script:taskProcess.Id) | Where-Object {[NativeUiTest]::GetDlgItem($_,901) -ne [IntPtr]::Zero}) | Select-Object -First 1}
function SettingsWindow {for($attempt=0;$attempt -lt 30;$attempt++){$found=@([NativeUiTest]::Windows($script:taskProcess.Id) | Where-Object {[NativeUiTest]::GetDlgItem($_,141) -ne [IntPtr]::Zero}) | Select-Object -First 1;if($found){return $found};Start-Sleep -Milliseconds 100};return $null}
function AboutWindow {for($attempt=0;$attempt -lt 30;$attempt++){$found=@([NativeUiTest]::Windows($script:taskProcess.Id) | Where-Object {[NativeUiTest]::GetDlgItem($_,144) -ne [IntPtr]::Zero}) | Select-Object -First 1;if($found){return $found};Start-Sleep -Milliseconds 100};return $null}
function SelectCombo([IntPtr]$Window,[int]$Id,[int]$Index){$h=[NativeUiTest]::GetDlgItem($Window,$Id);$null=[NativeUiTest]::SendMessage($h,0x14E,[IntPtr]$Index,[IntPtr]::Zero);$null=[NativeUiTest]::PostMessage($Window,0x111,[IntPtr]($Id+65536),$h);Start-Sleep -Milliseconds 700}
function Target([IntPtr]$Window){return [NativeUiTest]::GetProp($Window,'Lecoo.FanTarget').ToInt64()-1}
function Caption([IntPtr]$Window,[int]$Id){return [NativeUiTest]::ControlText([NativeUiTest]::GetDlgItem($Window,$Id))}
function Reopen {
 $second=Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru
 if(!$second.WaitForExit(10000)){throw 'Duplicate instance did not exit'}
 Start-Sleep -Seconds 1
 $window=MainWindow;if(!$window){throw 'Tray reopening failed'};return $window
}
function Capture([IntPtr]$Window,[string]$Name){
 $previous=[NativeUiTest]::SetThreadDpiAwarenessContext([IntPtr](-4))
 $null=[NativeUiTest]::SetForegroundWindow($Window);Start-Sleep -Milliseconds 250
 $r=[NativeUiTest+Rect]::new();$null=[NativeUiTest]::GetWindowRect($Window,[ref]$r)
 $bitmap=[Drawing.Bitmap]::new($r.Right-$r.Left,$r.Bottom-$r.Top)
 try{$graphics=[Drawing.Graphics]::FromImage($bitmap);try{$graphics.CopyFromScreen($r.Left,$r.Top,0,0,$bitmap.Size)}finally{$graphics.Dispose()};$bitmap.Save((Join-Path (Split-Path -Parent $ReportPath) ($Name+'.png')),[Drawing.Imaging.ImageFormat]::Png)}finally{$bitmap.Dispose();$null=[NativeUiTest]::SetThreadDpiAwarenessContext($previous)}
}
function Sample([string]$Phase){
 $inventory=@(Get-CimInstance Win32_Process);$rootId=$script:taskProcess.Id
 $ids=@($rootId)+@($inventory | Where-Object ParentProcessId -eq $rootId | ForEach-Object ProcessId)
 $counters=@(Get-CimInstance Win32_PerfFormattedData_PerfProc_Process)
 $rows=@(foreach($id in $ids){$p=Get-Process -Id $id -ErrorAction Stop;$c=$counters | Where-Object IDProcess -eq $id | Select-Object -First 1
  [pscustomobject]@{Pid=$id;WorkingSet=$p.WorkingSet64;PrivateCommit=$p.PrivateMemorySize64;PrivateWorkingSet=$c.WorkingSetPrivate;Gdi=[NativeUiTest]::GetGuiResources($p.Handle,0);User=[NativeUiTest]::GetGuiResources($p.Handle,1)}})
 return [pscustomobject]@{Phase=$Phase;Processes=$rows;PrivateWorkingSet=($rows | Measure-Object PrivateWorkingSet -Sum).Sum;PrivateCommit=($rows | Measure-Object PrivateCommit -Sum).Sum}
}
$taskProcess=$null;$taskInstance=$null;$taskOriginalMode=$null
$taskPreference=Join-Path $env:LOCALAPPDATA 'LecooRustPowerControl\preferences.txt'
$taskPreferenceExisted=Test-Path -LiteralPath $taskPreference
$taskPreferenceBytes=if($taskPreferenceExisted){[IO.File]::ReadAllBytes($taskPreference)}else{$null}
$taskThemeKey='HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
$taskThemeOriginal=Get-ItemProperty -LiteralPath $taskThemeKey
$taskThemeExisted=$taskThemeOriginal.PSObject.Properties.Name -contains 'AppsUseLightTheme'
$taskThemeValue=$taskThemeOriginal.AppsUseLightTheme
$taskReport=[ordered]@{SHA256=(Get-FileHash -LiteralPath $Executable).Hash;Passed=$false;Error=$null;Executable=$Executable;Checks=@();Memory=@();ResourceCycles=@();Restored=$false}
try{
 if(Get-Process lecoo-control-center -ErrorAction SilentlyContinue){throw 'Close existing app before native UI test'}
 $taskInstance=Get-CimInstance -Namespace root/WMI -ClassName PowerSwitchInterface | Where-Object InstanceName -eq 'ACPI\PNP0C14\IP3POWERSWITCH_0'
 $taskOriginalMode=[int](Invoke-CimMethod -InputObject $taskInstance -MethodName GetPowerMode).CurrentPowerMode
 New-Item -ItemType Directory -Path (Split-Path -Parent $taskPreference) -Force | Out-Null
 [IO.File]::WriteAllText($taskPreference,"close_behavior=ask`nlanguage=zh-CN`n")
 $taskProcess=Start-Process -FilePath $Executable -WindowStyle Normal -PassThru
 Start-Sleep -Seconds 4;$taskWindow=MainWindow;if(!$taskWindow){throw 'Native UI did not open'}
 $taskChildren=@([NativeUiTest]::Children($taskWindow));$taskCaptions=@($taskChildren | ForEach-Object {[NativeUiTest]::Text($_)})
 if($taskCaptions -match '原生|释放界面|后台保护继续运行|刷新状态|收进托盘|^退出$|重新连接|固件|本次请求|35%|100%'){throw 'Removed UI copy or button remains'}
 $taskReport.Checks+='All controls visible; removed copy and buttons absent'
 for($i=0;$i -lt 6;$i++){Start-Sleep -Seconds 2;$taskReport.Memory+=Sample 'Visible'}
 foreach($mode in @(2,0,1)){
  $id=@{2=110;0=111;1=112}[$mode];Click $taskWindow $id
  $actual=[int](Invoke-CimMethod -InputObject $taskInstance -MethodName GetPowerMode).CurrentPowerMode
  if($actual -ne $mode){throw 'Native profile switch readback mismatch'}
 }
 $taskReport.Checks+='Native profile buttons independently read back 2/0/1'
 $taskReport['ModeRowRects']=@(foreach($id in @(110,111,112)){$r=[NativeUiTest+Rect]::new();$null=[NativeUiTest]::GetWindowRect([NativeUiTest]::GetDlgItem($taskWindow,$id),[ref]$r);[pscustomobject]@{Id=$id;Top=$r.Top;Left=$r.Left}})
 if(@($taskReport.ModeRowRects.Top | Select-Object -Unique).Count -ne 1){throw 'Power options are not on the same row'}
 foreach($light in @(1,0)){
  Set-ItemProperty -LiteralPath $taskThemeKey -Name AppsUseLightTheme -Value $light
  $null=[NativeUiTest]::PostMessage($taskWindow,0x1A,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 3
  $titleDark=0;$null=[NativeUiTest]::DwmGetWindowAttribute($taskWindow,20,[ref]$titleDark,4)
  $pixel=[NativeUiTest]::Pixel($taskWindow);$expected=if($light -eq 1){0xF8F6F5}else{0x1C1816}
  if($pixel -ne $expected -or $titleDark -ne (1-$light)){throw "Theme did not follow system: pixel=$pixel title=$titleDark"}
  $name=if($light -eq 1){'light'}else{'dark'};Capture $taskWindow $name
  $taskReport.Checks+="System $name theme switches client area and title bar"
 }
 if((Caption $taskWindow 200) -notmatch '^[0-9.]+ °C$'){throw 'Thermal-zone reading unavailable'}
 $taskReport['ThermalZone']=Caption $taskWindow 200
 $taskReport['LayoutRects']=@(foreach($id in @(200,201,202,203,204,140,331)){$r=[NativeUiTest+Rect]::new();$null=[NativeUiTest]::GetWindowRect([NativeUiTest]::GetDlgItem($taskWindow,$id),[ref]$r);[pscustomobject]@{Id=$id;Top=$r.Top;Left=$r.Left;Right=$r.Right;Bottom=$r.Bottom}})
 Click $taskWindow 140;$settings=SettingsWindow;if(!$settings){throw 'Gear did not open settings'}
 # Focus/dropdown notifications must not clear or translate the native combo selection.
 $languageControl=[NativeUiTest]::GetDlgItem($settings,141)
 $null=[NativeUiTest]::PostMessage($settings,0x111,[IntPtr](141+7*65536),$languageControl);Start-Sleep -Milliseconds 300
 if([NativeUiTest]::SendMessage($languageControl,0x146,[IntPtr]::Zero,[IntPtr]::Zero).ToInt64() -ne 3){throw 'Language dropdown was changed by focus/dropdown event'}
 foreach($language in @(2,1)){
  SelectCombo $settings 141 $language
  $english=$language -eq 2
  $expected=if($english){'ACPI thermal zone'}else{'ACPI 热区温度'}
  if((Caption $taskWindow 300) -ne $expected){throw 'Language did not update main window immediately'}
  $expected=if($english){'Settings'}else{'设置'}
  if([NativeUiTest]::Text($settings) -ne $expected){throw 'Settings title not localized'}
  $name=if($english){'english'}else{'chinese'};Capture $taskWindow ($name+'-dark');Capture $settings ($name+'-settings-dark')
  Click $settings 143;$about=AboutWindow;if(!$about){throw 'About window did not open'}
  $expected=if($english){'Version 0.0.3'}else{'版本 0.0.3'}
  if((Caption $about 342) -ne $expected){throw 'About version not localized'}
  if((Caption $about 343) -notmatch ('© '+(Get-Date).Year+' ')){throw 'About copyright year is not current'}
  Capture $about ($name+'-about-dark')
  Click $about 146;$notices=@([NativeUiTest]::Windows($taskProcess.Id) | Where-Object {[NativeUiTest]::GetDlgItem($_,360) -ne [IntPtr]::Zero}) | Select-Object -First 1
  if(!$notices -or (Caption $notices 360) -notmatch 'Third-party notices'){throw 'Third-party notices did not load'}
  if([NativeUiTest]::SendMessage([NativeUiTest]::GetDlgItem($notices,360),0xBA,[IntPtr]::Zero,[IntPtr]::Zero).ToInt64() -lt 200){throw 'License document is truncated or lacks line breaks'}
  $null=[NativeUiTest]::PostMessage($notices,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 400
  if(![NativeUiTest]::IsWindowEnabled($about)){throw 'Closing notices left About disabled'}
  $null=[NativeUiTest]::PostMessage($about,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 400
  if(![NativeUiTest]::IsWindowEnabled($settings)){throw 'Closing About left Settings disabled'}
 }
 SelectCombo $settings 142 1
 if([IO.File]::ReadAllText($taskPreference) -notmatch '(?m)^close_behavior=tray$'){throw 'Settings close preference was not saved'}
 SelectCombo $settings 142 0
 if([IO.File]::ReadAllText($taskPreference) -notmatch '(?m)^close_behavior=ask$'){throw 'Ask every time close preference was not saved'}
 SelectCombo $settings 141 2
 $null=[NativeUiTest]::PostMessage($settings,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 500
 if(![NativeUiTest]::IsWindowEnabled($taskWindow)){throw 'Closing settings left dashboard disabled'}
 # Confirm language persists through an actual process restart.
 $null=[NativeUiTest]::PostMessage($taskWindow,0x111,[IntPtr]902,[IntPtr]::Zero)
 if(!$taskProcess.WaitForExit(10000)){throw 'Restart for language persistence failed'}
 $taskProcess=Start-Process -FilePath $Executable -WindowStyle Normal -PassThru
 Start-Sleep -Seconds 3;$taskWindow=MainWindow
 if((Caption $taskWindow 300) -ne 'ACPI thermal zone'){throw 'English did not persist after restart'}
 Set-ItemProperty -LiteralPath $taskThemeKey -Name AppsUseLightTheme -Value 1
 $null=[NativeUiTest]::PostMessage($taskWindow,0x1A,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 3
 Capture $taskWindow 'english-light'
 Click $taskWindow 140;$settings=SettingsWindow
 SelectCombo $settings 141 1
 Capture $taskWindow 'chinese-light';Capture $settings 'chinese-settings-light'
 Click $settings 143;$about=AboutWindow;Capture $about 'chinese-about-light'
 $null=[NativeUiTest]::PostMessage($about,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 400
 $null=[NativeUiTest]::PostMessage($settings,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 500
 $taskReport.Checks+='ACPI thermal-zone reading, compact grid, language switch/persistence, About/version/year and complete notices work'
 $slider=[NativeUiTest]::GetDlgItem($taskWindow,122)
 if([NativeUiTest]::IsWindowEnabled($slider) -or [NativeUiTest]::IsWindowEnabled([NativeUiTest]::GetDlgItem($taskWindow,123))){throw 'Reduced manual controls must be disabled'}
 Click $taskWindow 121;Start-Sleep -Seconds 3
 if((Target $taskWindow) -ne 100){throw 'Maximum fan request did not commit'}
 Click $taskWindow 140;$settings=SettingsWindow;Click $settings 143;$about=AboutWindow;Click $about 146
 $notices=@([NativeUiTest]::Windows($taskProcess.Id) | Where-Object {[NativeUiTest]::GetDlgItem($_,360) -ne [IntPtr]::Zero}) | Select-Object -First 1
 Start-Sleep -Seconds 4
 if((Target $taskWindow) -ne 100 -or [NativeUiTest]::GetProp($taskWindow,'Lecoo.FanReady').ToInt64() -ne 2){throw 'Settings/About/notices interrupted maximum fan heartbeat'}
 $null=[NativeUiTest]::PostMessage($notices,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 300
 $null=[NativeUiTest]::PostMessage($about,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 300
 $null=[NativeUiTest]::PostMessage($settings,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 400
 $taskReport.Checks+='Maximum fan heartbeat continues through Settings/About/notices'
 $taskReport.Checks+='Reduced manual controls disabled; maximum fan request available'
 $null=[NativeUiTest]::PostMessage($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 1
 $dialog=DialogWindow;if(!$dialog -or [NativeUiTest]::IsWindowEnabled($taskWindow)){throw 'Close choice dialog missing or owner not disabled'}
 Start-Sleep -Seconds 4
 if((Target $taskWindow) -ne 100 -or (Caption $taskWindow 206) -match 'RECOVERY'){throw 'Close dialog interrupted heartbeat'}
 Capture $dialog 'close-dialog';Click $dialog 901
 if([NativeUiTest]::IsWindow($taskWindow)){throw 'Tray choice did not destroy UI'}
 $taskReport.Memory+=Sample 'HiddenMaximum'
 $taskWindow=Reopen
 if($taskWindow -eq [IntPtr]::Zero){throw 'UI not recreated'}
 $taskReport.Checks+='Close dialog keeps maximum heartbeat alive; unremembered tray choice recreates UI'
 $null=[NativeUiTest]::PostMessage($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 1
 $dialog=DialogWindow;Click $dialog 903;Click $dialog 901
 if([IO.File]::ReadAllText($taskPreference) -notmatch '(?m)^close_behavior=tray$'){throw 'Remembered tray choice not saved'}
 $taskWindow=Reopen
 for($i=0;$i -lt 8;$i++){
  $null=[NativeUiTest]::ShowWindow($taskWindow,6);Start-Sleep -Milliseconds 400
  if([NativeUiTest]::IsWindow($taskWindow)){throw 'Minimize did not destroy UI'}
  $taskWindow=Reopen;$p=Get-Process -Id $taskProcess.Id
  $taskReport.ResourceCycles+=[pscustomobject]@{Cycle=$i;Gdi=[NativeUiTest]::GetGuiResources($p.Handle,0);User=[NativeUiTest]::GetGuiResources($p.Handle,1)}
 }
 if($taskReport.ResourceCycles[-1].Gdi -gt $taskReport.ResourceCycles[1].Gdi+3 -or $taskReport.ResourceCycles[-1].User -gt $taskReport.ResourceCycles[1].User+3){throw 'Window cycles leak GDI/USER handles'}
 $taskReport.Checks+='Remember tray; repeated minimize/reopen does not grow GDI/USER handles'
 $null=[NativeUiTest]::PostMessage($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 1
 if(DialogWindow){throw 'Remembered tray choice still prompted'}
 $taskWindow=Reopen;Click $taskWindow 140;$settings=SettingsWindow;SelectCombo $settings 142 0;$null=[NativeUiTest]::PostMessage($settings,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 500
 $null=[NativeUiTest]::PostMessage($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 1
 $dialog=DialogWindow;$null=[NativeUiTest]::PostMessage($dialog,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Milliseconds 500
 if(DialogWindow){throw 'Dialog cancellation did not close prompt'}
 if(![NativeUiTest]::IsWindowEnabled($taskWindow)){throw 'Dialog cancellation left owner disabled'}
 $null=[NativeUiTest]::PostMessage($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero);Start-Sleep -Seconds 1
 $dialog=DialogWindow;Click $dialog 903;Click $dialog 902
 if(!$taskProcess.WaitForExit(10000) -or $taskProcess.ExitCode -ne 0){throw 'Dialog exit did not shut down normally'}
 if([IO.File]::ReadAllText($taskPreference) -notmatch '(?m)^close_behavior=exit$'){throw 'Remembered exit choice not saved'}
 $taskReport.Checks+='Ask every time choice, cancel dialog and remember exit all work'
 $taskReport.Passed=$true
}catch{if($taskWindow){$taskReport['DashboardError']=Caption $taskWindow 206;Capture $taskWindow 'failure'};$taskReport.Error=$_.Exception.Message;$taskReport['ErrorLine']=$_.InvocationInfo.Line;$taskReport['ErrorStack']=$_.ScriptStackTrace}
finally{
 if($taskProcess -and !$taskProcess.HasExited){$taskProcess.Kill();$taskProcess.WaitForExit()}
 try{
  if($taskInstance){$result=Invoke-CimMethod -InputObject $taskInstance -MethodName SetFanControl -Arguments @{FanNumber=[byte]1;FanDuty=[byte]101};if($result.ResultStatus -eq 255){throw 'Final auto recovery failed'}
   $null=Invoke-CimMethod -InputObject $taskInstance -MethodName SetPowerMode -Arguments @{PowerMode=[byte]$taskOriginalMode}
   if([int](Invoke-CimMethod -InputObject $taskInstance -MethodName GetPowerMode).CurrentPowerMode -ne $taskOriginalMode){throw 'Mode restore failed'}}
  if($taskPreferenceExisted){[IO.File]::WriteAllBytes($taskPreference,$taskPreferenceBytes)}elseif(Test-Path -LiteralPath $taskPreference){Remove-Item -LiteralPath $taskPreference}
  if($taskThemeExisted){Set-ItemProperty -LiteralPath $taskThemeKey -Name AppsUseLightTheme -Value $taskThemeValue}else{Remove-ItemProperty -LiteralPath $taskThemeKey -Name AppsUseLightTheme}
  $taskReport.Restored=$true
 }catch{$taskReport.Passed=$false;$taskReport.Error=($taskReport.Error+'; '+$_.Exception.Message).Trim('; ')}
 $taskReport | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
}
Write-Output "Passed=$($taskReport.Passed), Report=$ReportPath"
if(!$taskReport.Passed){exit 1}
