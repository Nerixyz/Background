pub trait WinRectExt {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
}

impl WinRectExt for windows::Win32::Foundation::RECT {
    fn width(&self) -> i32 {
        self.right - self.left
    }

    fn height(&self) -> i32 {
        self.bottom - self.top
    }
}
