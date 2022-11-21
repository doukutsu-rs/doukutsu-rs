#[cfg(feature = "webbrowser")]
pub use webbrowser::open;

// stub for platforms webbrowser doesn't support, such as Horizon OS
#[cfg(not(feature = "webbrowser"))]
pub fn open(_url: &str) -> std::io::Result<()> {
    Ok(())
}
