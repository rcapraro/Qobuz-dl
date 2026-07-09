fn main() {
    // Embed the app icon into the Windows executable so the desktop/Start-Menu
    // shortcut, Explorer, and the pinned-taskbar icon all show it (shortcuts
    // read their icon from the .exe, not from the running window).
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
