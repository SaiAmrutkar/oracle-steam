// user32.dll proxy for Oracle Steam
// This DLL loads before Steam and injects our hook

use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, TRUE};
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

/// Clear Steam cache files BEFORE Steam loads them
/// This must happen early in DllMain before steamclient64.dll is loaded
fn clear_steam_cache_early() {
    // Clear appinfo.vdf to force Steam to rebuild library with our hooks
    let appinfo = r"C:\Program Files (x86)\Steam\appcache\appinfo.vdf";
    let packageinfo = r"C:\Program Files (x86)\Steam\appcache\packageinfo.vdf";
    
    // Delete files early before Steam opens handles
    let _ = std::fs::remove_file(appinfo);
    let _ = std::fs::remove_file(packageinfo);
}

static mut REAL_USER32: HINSTANCE = ptr::null_mut();
static mut ORACLE_HOOK: HINSTANCE = ptr::null_mut();
static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: DWORD,
    _lpv_reserved: LPVOID,
) -> BOOL {
    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            if INITIALIZED.swap(true, Ordering::SeqCst) {
                return TRUE;
            }
            
            // Clear Steam cache EARLY - before steamclient64.dll loads
            // This forces Steam to rebuild its library and call our hooks
            clear_steam_cache_early();
            
            // Load real user32.dll from System32
            let system32 = CString::new("C:\\Windows\\System32\\user32.dll").unwrap();
            REAL_USER32 = LoadLibraryA(system32.as_ptr());
            
            if REAL_USER32.is_null() {
                // Try SysWOW64 for 32-bit
                let syswow64 = CString::new("C:\\Windows\\SysWOW64\\user32.dll").unwrap();
                REAL_USER32 = LoadLibraryA(syswow64.as_ptr());
            }
            
            if REAL_USER32.is_null() {
                return 0; // FALSE - can't load real user32
            }
            
            // Load Oracle Hook DLL
            let hook_path = CString::new("C:\\Program Files (x86)\\Steam\\oracle_hook.dll").unwrap();
            ORACLE_HOOK = LoadLibraryA(hook_path.as_ptr());
            
            if !ORACLE_HOOK.is_null() {
                // Call DllMain to initialize
                type DllMainFn = unsafe extern "system" fn(HINSTANCE, DWORD, LPVOID) -> BOOL;
                let dll_main = GetProcAddress(ORACLE_HOOK, CString::new("DllMain").unwrap().as_ptr());
                if !dll_main.is_null() {
                    let dll_main_fn: DllMainFn = std::mem::transmute(dll_main);
                    let _ = dll_main_fn(ORACLE_HOOK, DLL_PROCESS_ATTACH, ptr::null_mut());
                }
            }
            
            TRUE
        }
        DLL_PROCESS_DETACH => {
            TRUE
        }
        _ => TRUE,
    }
}

// Export forwarding macros
// These forward calls to the real user32.dll

#[macro_export]
macro_rules! forward_func {
    ($name:ident, ($($param:ident: $ty:ty),*), $ret:ty) => {
        #[no_mangle]
        pub unsafe extern "system" fn $name($($param: $ty),*) -> $ret {
            static mut REAL_FUNC: Option<unsafe extern "system" fn($($ty),*) -> $ret> = None;
            
            if REAL_FUNC.is_none() && !REAL_USER32.is_null() {
                let name = CString::new(stringify!($name)).unwrap();
                let addr = GetProcAddress(REAL_USER32, name.as_ptr());
                if !addr.is_null() {
                    REAL_FUNC = Some(std::mem::transmute(addr));
                }
            }
            
            if let Some(func) = REAL_FUNC {
                func($($param),*)
            } else {
                std::mem::zeroed()
            }
        }
    };
}

// Forward common user32 functions
forward_func!(MessageBoxA, (hWnd: HINSTANCE, lpText: *const i8, lpCaption: *const i8, uType: u32), i32);
forward_func!(MessageBoxW, (hWnd: HINSTANCE, lpText: *const u16, lpCaption: *const u16, uType: u32), i32);
forward_func!(CreateWindowExA, (dwExStyle: u32, lpClassName: *const i8, lpWindowName: *const i8, dwStyle: u32, X: i32, Y: i32, nWidth: i32, nHeight: i32, hWndParent: HINSTANCE, hMenu: HINSTANCE, hInstance: HINSTANCE, lpParam: LPVOID), HINSTANCE);
forward_func!(CreateWindowExW, (dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16, dwStyle: u32, X: i32, Y: i32, nWidth: i32, nHeight: i32, hWndParent: HINSTANCE, hMenu: HINSTANCE, hInstance: HINSTANCE, lpParam: LPVOID), HINSTANCE);
forward_func!(RegisterClassA, (lpWndClass: *const u8), u16);
forward_func!(RegisterClassW, (lpWndClass: *const u8), u16);
forward_func!(RegisterClassExA, (lpWndClass: *const u8), u16);
forward_func!(RegisterClassExW, (lpWndClass: *const u8), u16);
forward_func!(DefWindowProcA, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(DefWindowProcW, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(GetMessageA, (lpMsg: *mut u8, hWnd: HINSTANCE, wMsgFilterMin: u32, wMsgFilterMax: u32), BOOL);
forward_func!(GetMessageW, (lpMsg: *mut u8, hWnd: HINSTANCE, wMsgFilterMin: u32, wMsgFilterMax: u32), BOOL);
forward_func!(PeekMessageA, (lpMsg: *mut u8, hWnd: HINSTANCE, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32), BOOL);
forward_func!(PeekMessageW, (lpMsg: *mut u8, hWnd: HINSTANCE, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32), BOOL);
forward_func!(TranslateMessage, (lpMsg: *const u8), BOOL);
forward_func!(DispatchMessageA, (lpMsg: *const u8), isize);
forward_func!(DispatchMessageW, (lpMsg: *const u8), isize);
forward_func!(SendMessageA, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(SendMessageW, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(PostMessageA, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), BOOL);
forward_func!(PostMessageW, (hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), BOOL);
forward_func!(PostQuitMessage, (nExitCode: i32), ());
forward_func!(CallWindowProcA, (lpPrevWndFunc: *const u8, hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(CallWindowProcW, (lpPrevWndFunc: *const u8, hWnd: HINSTANCE, Msg: u32, wParam: usize, lParam: isize), isize);
forward_func!(SetWindowLongA, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), i32);
forward_func!(SetWindowLongW, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), i32);
forward_func!(GetWindowLongA, (hWnd: HINSTANCE, nIndex: i32), isize);
forward_func!(GetWindowLongW, (hWnd: HINSTANCE, nIndex: i32), isize);
forward_func!(SetWindowLongPtrA, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), isize);
forward_func!(SetWindowLongPtrW, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), isize);
forward_func!(GetWindowLongPtrA, (hWnd: HINSTANCE, nIndex: i32), isize);
forward_func!(GetWindowLongPtrW, (hWnd: HINSTANCE, nIndex: i32), isize);
forward_func!(ShowWindow, (hWnd: HINSTANCE, nCmdShow: i32), BOOL);
forward_func!(UpdateWindow, (hWnd: HINSTANCE), BOOL);
forward_func!(InvalidateRect, (hWnd: HINSTANCE, lpRect: *const u8, bErase: BOOL), BOOL);
forward_func!(ValidateRect, (hWnd: HINSTANCE, lpRect: *const u8), BOOL);
forward_func!(GetClientRect, (hWnd: HINSTANCE, lpRect: *mut u8), BOOL);
forward_func!(GetWindowRect, (hWnd: HINSTANCE, lpRect: *mut u8), BOOL);
forward_func!(ClientToScreen, (hWnd: HINSTANCE, lpPoint: *mut u8), BOOL);
forward_func!(ScreenToClient, (hWnd: HINSTANCE, lpPoint: *mut u8), BOOL);
forward_func!(SetCursor, (hCursor: HINSTANCE), HINSTANCE);
forward_func!(GetCursor, (), HINSTANCE);
forward_func!(LoadCursorA, (hInstance: HINSTANCE, lpCursorName: *const i8), HINSTANCE);
forward_func!(LoadCursorW, (hInstance: HINSTANCE, lpCursorName: *const u16), HINSTANCE);
forward_func!(LoadIconA, (hInstance: HINSTANCE, lpIconName: *const i8), HINSTANCE);
forward_func!(LoadIconW, (hInstance: HINSTANCE, lpIconName: *const u16), HINSTANCE);
forward_func!(LoadImageA, (hInst: HINSTANCE, lpszName: *const i8, uType: u32, cxDesired: i32, cyDesired: i32, fuLoad: u32), HINSTANCE);
forward_func!(LoadImageW, (hInst: HINSTANCE, lpszName: *const u16, uType: u32, cxDesired: i32, cyDesired: i32, fuLoad: u32), HINSTANCE);
forward_func!(LoadBitmapA, (hInstance: HINSTANCE, lpBitmapName: *const i8), HINSTANCE);
forward_func!(LoadBitmapW, (hInstance: HINSTANCE, lpBitmapName: *const u16), HINSTANCE);
forward_func!(SetTimer, (hWnd: HINSTANCE, nIDEvent: usize, uElapse: u32, lpTimerFunc: *const u8), usize);
forward_func!(KillTimer, (hWnd: HINSTANCE, uIDEvent: usize), BOOL);
forward_func!(SetCapture, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(ReleaseCapture, (), BOOL);
forward_func!(GetCapture, (), HINSTANCE);
forward_func!(GetWindowDC, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(GetDC, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(ReleaseDC, (hWnd: HINSTANCE, hDC: HINSTANCE), i32);
forward_func!(BeginPaint, (hWnd: HINSTANCE, lpPaint: *mut u8), HINSTANCE);
forward_func!(EndPaint, (hWnd: HINSTANCE, lpPaint: *const u8), BOOL);
forward_func!(GetSysColor, (nIndex: i32), u32);
forward_func!(GetSysColorBrush, (nIndex: i32), HINSTANCE);
forward_func!(FillRect, (hDC: HINSTANCE, lprc: *const u8, hbr: HINSTANCE), i32);
forward_func!(DrawTextA, (hDC: HINSTANCE, lpchText: *const i8, cchText: i32, lprc: *mut u8, format: u32), i32);
forward_func!(DrawTextW, (hDC: HINSTANCE, lpchText: *const u16, cchText: i32, lprc: *mut u8, format: u32), i32);
forward_func!(SetBkMode, (hDC: HINSTANCE, mode: i32), i32);
forward_func!(SetBkColor, (hDC: HINSTANCE, color: u32), u32);
forward_func!(SetTextColor, (hDC: HINSTANCE, color: u32), u32);
forward_func!(GetDlgItem, (hDlg: HINSTANCE, nIDDlgItem: i32), HINSTANCE);
forward_func!(SetDlgItemTextA, (hDlg: HINSTANCE, nIDDlgItem: i32, lpString: *const i8), BOOL);
forward_func!(SetDlgItemTextW, (hDlg: HINSTANCE, nIDDlgItem: i32, lpString: *const u16), BOOL);
forward_func!(GetDlgItemTextA, (hDlg: HINSTANCE, nIDDlgItem: i32, lpString: *mut i8, cchMax: i32), u32);
forward_func!(GetDlgItemTextW, (hDlg: HINSTANCE, nIDDlgItem: i32, lpString: *mut u16, cchMax: i32), u32);
forward_func!(CheckDlgButton, (hDlg: HINSTANCE, nIDButton: i32, uCheck: u32), BOOL);
forward_func!(IsDlgButtonChecked, (hDlg: HINSTANCE, nIDButton: i32), u32);
forward_func!(EnableWindow, (hWnd: HINSTANCE, bEnable: BOOL), BOOL);
forward_func!(IsWindowEnabled, (hWnd: HINSTANCE), BOOL);
forward_func!(IsWindowVisible, (hWnd: HINSTANCE), BOOL);
forward_func!(SetFocus, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(GetFocus, (), HINSTANCE);
forward_func!(SetActiveWindow, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(GetActiveWindow, (), HINSTANCE);
forward_func!(SetForegroundWindow, (hWnd: HINSTANCE), BOOL);
forward_func!(GetForegroundWindow, (), HINSTANCE);
forward_func!(IsWindow, (hWnd: HINSTANCE), BOOL);
forward_func!(IsChild, (hWndParent: HINSTANCE, hWnd: HINSTANCE), BOOL);
forward_func!(DestroyWindow, (hWnd: HINSTANCE), BOOL);
forward_func!(CloseWindow, (hWnd: HINSTANCE), BOOL);
forward_func!(MoveWindow, (hWnd: HINSTANCE, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: BOOL), BOOL);
forward_func!(SetWindowPos, (hWnd: HINSTANCE, hWndInsertAfter: HINSTANCE, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32), BOOL);
forward_func!(GetWindowTextA, (hWnd: HINSTANCE, lpString: *mut i8, nMaxCount: i32), i32);
forward_func!(GetWindowTextW, (hWnd: HINSTANCE, lpString: *mut u16, nMaxCount: i32), i32);
forward_func!(SetWindowTextA, (hWnd: HINSTANCE, lpString: *const i8), BOOL);
forward_func!(SetWindowTextW, (hWnd: HINSTANCE, lpString: *const u16), BOOL);
forward_func!(FindWindowA, (lpClassName: *const i8, lpWindowName: *const i8), HINSTANCE);
forward_func!(FindWindowW, (lpClassName: *const u16, lpWindowName: *const u16), HINSTANCE);
forward_func!(FindWindowExA, (hWndParent: HINSTANCE, hWndChildAfter: HINSTANCE, lpszClass: *const i8, lpszWindow: *const i8), HINSTANCE);
forward_func!(FindWindowExW, (hWndParent: HINSTANCE, hWndChildAfter: HINSTANCE, lpszClass: *const u16, lpszWindow: *const u16), HINSTANCE);
forward_func!(EnumWindows, (lpEnumFunc: *const u8, lParam: isize), BOOL);
forward_func!(EnumChildWindows, (hWndParent: HINSTANCE, lpEnumFunc: *const u8, lParam: isize), BOOL);
forward_func!(GetDesktopWindow, (), HINSTANCE);
forward_func!(GetParent, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(SetParent, (hWndChild: HINSTANCE, hWndNewParent: HINSTANCE), HINSTANCE);
forward_func!(GetWindow, (hWnd: HINSTANCE, uCmd: u32), HINSTANCE);
forward_func!(GetClassNameA, (hWnd: HINSTANCE, lpClassName: *mut i8, nMaxCount: i32), i32);
forward_func!(GetClassNameW, (hWnd: HINSTANCE, lpClassName: *mut u16, nMaxCount: i32), i32);
forward_func!(RegisterRawInputDevices, (pRawInputDevices: *const u8, uiNumDevices: u32, cbSize: u32), BOOL);
forward_func!(GetRawInputData, (hRawInput: HINSTANCE, uiCommand: u32, pData: *mut u8, pcbSize: *mut u32, cbSizeHeader: u32), u32);
forward_func!(GetKeyState, (nVirtKey: i32), i16);
forward_func!(GetAsyncKeyState, (vKey: i32), i16);
forward_func!(GetKeyboardState, (lpKeyState: *mut u8), BOOL);
forward_func!(SetKeyboardState, (lpKeyState: *const u8), BOOL);
forward_func!(GetKeyNameTextA, (lParam: isize, lpString: *mut i8, cchSize: i32), i32);
forward_func!(GetKeyNameTextW, (lParam: isize, lpString: *mut u16, cchSize: i32), i32);
forward_func!(MapVirtualKeyA, (uCode: u32, uMapType: u32), u32);
forward_func!(MapVirtualKeyW, (uCode: u32, uMapType: u32), u32);
forward_func!(MapVirtualKeyExA, (uCode: u32, uMapType: u32, dwhkl: HINSTANCE), u32);
forward_func!(MapVirtualKeyExW, (uCode: u32, uMapType: u32, dwhkl: HINSTANCE), u32);
forward_func!(ToUnicode, (wVirtKey: u32, wScanCode: u32, lpKeyState: *const u8, pwszBuff: *mut u16, cchBuff: i32, wFlags: u32), i32);
forward_func!(ToAscii, (uVirtKey: u32, uScanCode: u32, lpKeyState: *const u8, lpChar: *mut u16, uFlags: u32), i32);
forward_func!(VkKeyScanA, (ch: i8), i16);
forward_func!(VkKeyScanW, (ch: u16), i16);
forward_func!(VkKeyScanExA, (ch: i8, dwhkl: HINSTANCE), i16);
forward_func!(VkKeyScanExW, (ch: u16, dwhkl: HINSTANCE), i16);
forward_func!(TrackMouseEvent, (lpEventTrack: *mut u8), BOOL);
forward_func!(GetWindowThreadProcessId, (hWnd: HINSTANCE, lpdwProcessId: *mut u32), u32);
forward_func!(GetClassInfoA, (hInstance: HINSTANCE, lpClassName: *const i8, lpWndClass: *mut u8), BOOL);
forward_func!(GetClassInfoW, (hInstance: HINSTANCE, lpClassName: *const u16, lpWndClass: *mut u8), BOOL);
forward_func!(GetClassInfoExA, (hInstance: HINSTANCE, lpszClass: *const i8, lpwcx: *mut u8), BOOL);
forward_func!(GetClassInfoExW, (hInstance: HINSTANCE, lpszClass: *const u16, lpwcx: *mut u8), BOOL);
forward_func!(WaitMessage, (), BOOL);
forward_func!(WaitForInputIdle, (hProcess: HINSTANCE, dwMilliseconds: u32), u32);
forward_func!(ChildWindowFromPoint, (hWndParent: HINSTANCE, point: isize), HINSTANCE);
forward_func!(ChildWindowFromPointEx, (hWnd: HINSTANCE, pt: isize, flags: u32), HINSTANCE);
forward_func!(WindowFromPoint, (point: isize), HINSTANCE);
forward_func!(RealChildWindowFromPoint, (hWndParent: HINSTANCE, ptParentClient: isize), HINSTANCE);
forward_func!(SetWindowContextHelpId, (hWnd: HINSTANCE, dwContextHelpId: u32), BOOL);
forward_func!(GetWindowContextHelpId, (hWnd: HINSTANCE), u32);
forward_func!(SetMenu, (hWnd: HINSTANCE, hMenu: HINSTANCE), BOOL);
forward_func!(GetMenu, (hWnd: HINSTANCE), HINSTANCE);
forward_func!(DrawMenuBar, (hWnd: HINSTANCE), BOOL);
forward_func!(GetSystemMenu, (hWnd: HINSTANCE, bRevert: BOOL), HINSTANCE);
forward_func!(AppendMenuA, (hMenu: HINSTANCE, uFlags: u32, uIDNewItem: usize, lpNewItem: *const i8), BOOL);
forward_func!(AppendMenuW, (hMenu: HINSTANCE, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16), BOOL);
forward_func!(InsertMenuA, (hMenu: HINSTANCE, uPosition: u32, uFlags: u32, uIDNewItem: usize, lpNewItem: *const i8), BOOL);
forward_func!(InsertMenuW, (hMenu: HINSTANCE, uPosition: u32, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16), BOOL);
forward_func!(DeleteMenu, (hMenu: HINSTANCE, uPosition: u32, uFlags: u32), BOOL);
forward_func!(RemoveMenu, (hMenu: HINSTANCE, uPosition: u32, uFlags: u32), BOOL);
forward_func!(EnableMenuItem, (hMenu: HINSTANCE, uIDEnableItem: u32, uEnable: u32), BOOL);
forward_func!(CheckMenuItem, (hMenu: HINSTANCE, uIDCheckItem: u32, uCheck: u32), u32);
forward_func!(CheckMenuRadioItem, (hMenu: HINSTANCE, idFirst: u32, idLast: u32, idCheck: u32, uFlags: u32), BOOL);
forward_func!(GetSubMenu, (hMenu: HINSTANCE, nPos: i32), HINSTANCE);
forward_func!(GetMenuItemID, (hMenu: HINSTANCE, nPos: i32), u32);
forward_func!(GetMenuItemCount, (hMenu: HINSTANCE), i32);
forward_func!(GetMenuStringA, (hMenu: HINSTANCE, uIDItem: u32, lpString: *mut i8, cchMax: i32, uFlag: u32), i32);
forward_func!(GetMenuStringW, (hMenu: HINSTANCE, uIDItem: u32, lpString: *mut u16, cchMax: i32, uFlag: u32), i32);
forward_func!(GetMenuState, (hMenu: HINSTANCE, uId: u32, uFlags: u32), u32);
forward_func!(SetMenuItemBitmaps, (hMenu: HINSTANCE, uPosition: u32, uFlags: u32, hBitmapUnchecked: HINSTANCE, hBitmapChecked: HINSTANCE), BOOL);
forward_func!(GetMenuCheckMarkDimensions, (), isize);
forward_func!(DrawStateA, (hdc: HINSTANCE, hbrFore: HINSTANCE, qfnCallBack: *const u8, lData: isize, wData: isize, x: i32, y: i32, cx: i32, cy: i32, uFlags: u32), BOOL);
forward_func!(DrawStateW, (hdc: HINSTANCE, hbrFore: HINSTANCE, qfnCallBack: *const u8, lData: isize, wData: isize, x: i32, y: i32, cx: i32, cy: i32, uFlags: u32), BOOL);
forward_func!(DrawFrameControl, (hdc: HINSTANCE, lprc: *mut u8, uType: u32, uState: u32), BOOL);
forward_func!(DrawEdge, (hdc: HINSTANCE, qrc: *mut u8, edge: u32, grfFlags: u32), BOOL);
forward_func!(DrawFocusRect, (hDC: HINSTANCE, lprc: *const u8), BOOL);
forward_func!(DrawCaption, (hwnd: HINSTANCE, hdc: HINSTANCE, lprect: *const u8, flags: u32), BOOL);
forward_func!(DrawAnimatedRects, (hwnd: HINSTANCE, idAni: i32, lprcFrom: *const u8, lprcTo: *const u8), BOOL);
forward_func!(GetMessagePos, (), u32);
forward_func!(GetMessageTime, (), i32);
forward_func!(GetMessageExtraInfo, (), isize);
forward_func!(SetMessageExtraInfo, (lParam: isize), isize);
forward_func!(IsDialogMessageA, (hDlg: HINSTANCE, lpMsg: *const u8), BOOL);
forward_func!(IsDialogMessageW, (hDlg: HINSTANCE, lpMsg: *const u8), BOOL);
forward_func!(MapDialogRect, (hDlg: HINSTANCE, lpRect: *mut u8), BOOL);
forward_func!(DlgDirListA, (hDlg: HINSTANCE, lpPathSpec: *mut i8, nIDListBox: i32, nIDStaticPath: i32, uFileType: u32), i32);
forward_func!(DlgDirListW, (hDlg: HINSTANCE, lpPathSpec: *mut u16, nIDListBox: i32, nIDStaticPath: i32, uFileType: u32), i32);
forward_func!(DlgDirSelectExA, (hDlg: HINSTANCE, lpString: *mut i8, nCount: i32, nIDListBox: i32), BOOL);
forward_func!(DlgDirSelectExW, (hDlg: HINSTANCE, lpString: *mut u16, nCount: i32, nIDListBox: i32), BOOL);
forward_func!(DlgDirListComboBoxA, (hDlg: HINSTANCE, lpPathSpec: *mut i8, nIDComboBox: i32, nIDStaticPath: i32, uFileType: u32), i32);
forward_func!(DlgDirListComboBoxW, (hDlg: HINSTANCE, lpPathSpec: *mut u16, nIDComboBox: i32, nIDStaticPath: i32, uFileType: u32), i32);
forward_func!(DlgDirSelectComboBoxExA, (hDlg: HINSTANCE, lpString: *mut i8, nCount: i32, nIDComboBox: i32), BOOL);
forward_func!(DlgDirSelectComboBoxExW, (hDlg: HINSTANCE, lpString: *mut u16, nCount: i32, nIDComboBox: i32), BOOL);
forward_func!(OpenClipboard, (hWndNewOwner: HINSTANCE), BOOL);
forward_func!(CloseClipboard, (), BOOL);
forward_func!(GetClipboardOwner, (), HINSTANCE);
forward_func!(SetClipboardData, (uFormat: u32, hMem: HINSTANCE), HINSTANCE);
forward_func!(GetClipboardData, (uFormat: u32), HINSTANCE);
forward_func!(IsClipboardFormatAvailable, (format: u32), BOOL);
forward_func!(EmptyClipboard, (), BOOL);
forward_func!(EnumClipboardFormats, (format: u32), u32);
forward_func!(CountClipboardFormats, (), i32);
forward_func!(RegisterClipboardFormatA, (lpszFormat: *const i8), u32);
forward_func!(RegisterClipboardFormatW, (lpszFormat: *const u16), u32);
forward_func!(GetClipboardFormatNameA, (format: u32, lpszFormatName: *mut i8, cchMaxCount: i32), i32);
forward_func!(GetClipboardFormatNameW, (format: u32, lpszFormatName: *mut u16, cchMaxCount: i32), i32);
forward_func!(SetClipboardViewer, (hWndNewViewer: HINSTANCE), HINSTANCE);
forward_func!(GetClipboardViewer, (), HINSTANCE);
forward_func!(ChangeClipboardChain, (hWndRemove: HINSTANCE, hWndNewNext: HINSTANCE), BOOL);
forward_func!(SetClassLongA, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), u32);
forward_func!(SetClassLongW, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), u32);
forward_func!(GetClassLongA, (hWnd: HINSTANCE, nIndex: i32), u32);
forward_func!(GetClassLongW, (hWnd: HINSTANCE, nIndex: i32), u32);
forward_func!(SetClassLongPtrA, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), usize);
forward_func!(SetClassLongPtrW, (hWnd: HINSTANCE, nIndex: i32, dwNewLong: isize), usize);
forward_func!(GetClassLongPtrA, (hWnd: HINSTANCE, nIndex: i32), usize);
forward_func!(GetClassLongPtrW, (hWnd: HINSTANCE, nIndex: i32), usize);
