use anyhow::{Context, bail};
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, RDW_ALLCHILDREN,
        RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RedrawWindow,
    },
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{
            FindWindowExW, FindWindowW, GWL_EXSTYLE, GWL_STYLE, GetWindowLongW, GetWindowRect,
            HWND_TOP, SMTO_NORMAL, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageTimeoutW, SetParent,
            SetWindowLongPtrW, SetWindowLongW, SetWindowPos, WS_CHILD, WS_EX_LEFT,
            WS_EX_LTRREADING, WS_EX_NOACTIVATE, WS_EX_RIGHTSCROLLBAR, WS_EX_TOOLWINDOW,
            WS_EX_TRANSPARENT, WS_POPUP, WS_VISIBLE,
        },
    },
};
use windows_strings::w;

use crate::{config::CONFIG, extensions::WinRectExt};

pub fn setup_for_hwnd(hwnd: HWND) -> anyhow::Result<()> {
    let workerw = find_workerw().context("find worker")?;

    unsafe {
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            (WS_EX_LEFT
                | WS_EX_LTRREADING
                | WS_EX_RIGHTSCROLLBAR
                | WS_EX_NOACTIVATE
                | WS_EX_TOOLWINDOW
                | WS_EX_TRANSPARENT)
                .0 as _,
        );
        SetWindowLongPtrW(hwnd, GWL_STYLE, WS_POPUP.0 as _);
    }

    let monitor = find_monitor_rect()?;

    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            monitor.left,
            monitor.top,
            monitor.width(),
            monitor.height(),
            SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        let mut old_rect = RECT::default();
        GetWindowRect(workerw, &raw mut old_rect)?;

        SetParent(hwnd, Some(workerw))?;

        let mut parent_rect = RECT::default();
        GetWindowRect(workerw, &raw mut parent_rect)?;

        let my_scale = GetDpiForWindow(hwnd);
        if my_scale == 0 {
            bail!("invalid scale");
        }
        SetWindowLongW(
            hwnd,
            GWL_STYLE,
            GetWindowLongW(hwnd, GWL_STYLE) | WS_CHILD.0 as i32 | WS_VISIBLE.0 as i32,
        );
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_ERASE | RDW_INVALIDATE | RDW_FRAME | RDW_ALLCHILDREN,
        );
    }

    Ok(())
}

// This is mostly ported from https://github.com/rocksdanister/lively/blob/7ab3587b916cdeb023e5c07b4ff30ca76ac0a5c7/src/Lively/Lively/Core/WinDesktopCore.cs
fn find_workerw() -> windows::core::Result<HWND> {
    unsafe {
        // + Progman
        //   +-- SHELLDLL_DefView

        let progman = FindWindowW(w!("Progman"), None)?;
        // Spawn WorkerW
        let _ = SendMessageTimeoutW(
            progman,
            0x52c,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            None,
        );

        // let def_view = FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), None)?;
        let worker_w = FindWindowExW(Some(progman), None, w!("WorkerW"), None)?;

        Ok(worker_w)
        // WorkerW is somewhere else on pre-win11-24H2 but I don't use that version
    }
}

fn find_monitor_rect() -> anyhow::Result<RECT> {
    let (x, y) = CONFIG.monitor_at_pos();
    unsafe {
        let hmon = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        if hmon.is_invalid() {
            bail!("Failed to find monitor at ({x}, {y})");
        }

        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmon, &raw mut info as *mut _).as_bool() {
            bail!("Failed to get monitor info");
        }

        Ok(info.rcMonitor)
    }
}
