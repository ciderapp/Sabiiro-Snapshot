#![cfg(target_os = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::ffi::c_void;
pub use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Dwm::*, Gdi::*},
    System::{LibraryLoader::*, SystemInformation::*},
};

use super::common::Color;

pub fn apply_blur(hwnd: HWND, color: Option<Color>) -> bool {
    if is_win7() {
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: true.into(),
            hRgnBlur: HRGN::default(),
            fTransitionOnMaximized: 0,
        };
        unsafe {
            let _ = DwmEnableBlurBehindWindow(hwnd, &bb);
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_ENABLE_BLURBEHIND, color);
        }
    } else {
        return false;
    }

    true
}

pub fn clear_blur(hwnd: HWND) -> bool {
    if is_win7() {
        let bb = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE,
            fEnable: false.into(),
            hRgnBlur: HRGN::default(),
            fTransitionOnMaximized: 0,
        };
        unsafe {
            let _ = DwmEnableBlurBehindWindow(hwnd, &bb);
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None);
        }
    } else {
        return false;
    }
    true
}

pub fn apply_acrylic(hwnd: HWND, color: Option<Color>) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as *const _ as _,
                4,
            );
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(
                hwnd,
                ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND,
                color,
            );
        }
    } else {
        return false;
    }
    true
}

pub fn clear_acrylic(hwnd: HWND) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _,
                4,
            );
        }
    } else if is_swca_supported() {
        unsafe {
            SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None);
        }
    } else {
        return false;
    }
    true
}

pub fn apply_mica(hwnd: HWND) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as *const _ as _,
                4,
            );
        }
    } else if is_undocumented_mica_supported() {
        unsafe {
            DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
        }
    } else {
        return false;
    }
    true
}

pub fn clear_mica(hwnd: HWND) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _,
                4,
            );
        }
    } else if is_undocumented_mica_supported() {
        unsafe {
            DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &0 as *const _ as _, 4);
        }
    } else {
        return false;
    }
    true
}

pub fn set_light_mode(hwnd: HWND) -> bool {
    if is_atleast_win10() {
        unsafe {
            let _ = DwmSetWindowAttribute(hwnd, 20, &0 as *const _ as _, 4);
        }
        return true;
    }
    false
}

pub fn set_dark_mode(hwnd: HWND) -> bool {
    if is_atleast_win10() {
        unsafe {
            let _ = DwmSetWindowAttribute(hwnd, 20, &1 as *const _ as _, 4);
        }
        return true;
    }
    false
}

pub fn apply_tabbed(hwnd: HWND) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TABBEDWINDOW as *const _ as _,
                4,
            );
        }
    } else if is_tabbed_supported() {
        unsafe {
            DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
        }
    } else {
        return false;
    }
    true
}

pub fn clear_tabbed(hwnd: HWND) -> bool {
    if is_backdroptype_supported() {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE,
                &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _,
                4,
            );
        }
    } else if is_tabbed_supported() {
        unsafe {
            DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &0 as *const _ as _, 4);
        }
    } else {
        return false;
    }
    true
}

fn get_function_impl(library: &str, function: &str) -> Option<FARPROC> {
    assert_eq!(library.chars().last(), Some('\0'));
    assert_eq!(function.chars().last(), Some('\0'));

    let module = unsafe { LoadLibraryA(library.as_ptr()) };
    if module == 0 {
        return None;
    }
    Some(unsafe { GetProcAddress(module, function.as_ptr()) })
}

macro_rules! get_function {
    ($lib:expr, $func:ident) => {
        get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0'))
            .map(|f| std::mem::transmute::<::windows_sys::Win32::Foundation::FARPROC, $func>(f))
    };
}

/// Returns a tuple of (major, minor, buildnumber)
fn get_windows_ver() -> Option<(u32, u32, u32)> {
    type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
    let handle = unsafe { get_function!("ntdll.dll", RtlGetVersion) };
    handle.and_then(|rtl_get_version| unsafe {
        let mut vi = OSVERSIONINFOW {
            dwOSVersionInfoSize: 0,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };

        let status = (rtl_get_version)(&mut vi as _);

        if status >= 0 {
            Some((vi.dwMajorVersion, vi.dwMinorVersion, vi.dwBuildNumber))
        } else {
            None
        }
    })
}

#[repr(C)]
struct ACCENT_POLICY {
    AccentState: u32,
    AccentFlags: u32,
    GradientColor: u32,
    AnimationId: u32,
}

type WindowCompositionAttrib = u32;

#[repr(C)]
struct WindowCompositionAttribData {
    Attrib: WindowCompositionAttrib,
    pvData: *mut c_void,
    cbData: usize,
}

#[derive(PartialEq)]
#[repr(C)]
enum ACCENT_STATE {
    ACCENT_DISABLED = 0,
    ACCENT_ENABLE_BLURBEHIND = 3,
    ACCENT_ENABLE_ACRYLICBLURBEHIND = 4,
}

unsafe fn SetWindowCompositionAttribute(
    hwnd: HWND,
    accent_state: ACCENT_STATE,
    color: Option<Color>,
) {
    type SetWindowCompositionAttribute =
        unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> BOOL;

    if let Some(set_window_composition_attribute) =
        get_function!("user32.dll", SetWindowCompositionAttribute)
    {
        let mut color = color.unwrap_or_default();

        let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
        if is_acrylic && color.3 == 0 {
            // acrylic doesn't like to have 0 alpha
            color.3 = 1;
        }

        let mut policy = ACCENT_POLICY {
            AccentState: accent_state as _,
            AccentFlags: if is_acrylic { 0 } else { 2 },
            GradientColor: (color.0 as u32)
                | (color.1 as u32) << 8
                | (color.2 as u32) << 16
                | (color.3 as u32) << 24,
            AnimationId: 0,
        };

        let mut data = WindowCompositionAttribData {
            Attrib: 0x13,
            pvData: &mut policy as *mut _ as _,
            cbData: std::mem::size_of_val(&policy),
        };

        set_window_composition_attribute(hwnd, &mut data as *mut _ as _);
    }
}

const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = 1029i32;
const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38i32;

#[allow(unused)]
#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
    DWMSBT_DISABLE = 1,         // None
    DWMSBT_MAINWINDOW = 2,      // Mica
    DWMSBT_TRANSIENTWINDOW = 3, // Acrylic
    DWMSBT_TABBEDWINDOW = 4,    // Tabbed
}

fn is_win7() -> bool {
    let v = get_windows_ver().unwrap_or_default();
    v.0 == 6 && v.1 == 1
}

fn is_atleast_win10() -> bool {
    let v = get_windows_ver().unwrap_or_default();
    v.0 == 10 && v.2 >= 17763
}

fn is_swca_supported() -> bool {
    is_at_least_build(17763)
}

fn is_undocumented_mica_supported() -> bool {
    is_at_least_build(22000)
}

fn is_tabbed_supported() -> bool {
    is_at_least_build(22621)
}

fn is_backdroptype_supported() -> bool {
    is_at_least_build(22523)
}

pub fn is_at_least_build(build: u32) -> bool {
    let v = get_windows_ver().unwrap_or_default();
    v.2 >= build
}
