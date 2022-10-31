// This [build.rs] is for setting Windows icons.
// The icon in [File Explorer] gets set here.
// The icon in the taskbar and top of the App window gets
// set in [src/main.rs, src/constants.rs] at runtime with
// pre-compiled bytes using [include_bytes!()] on the images in [images/].
fn main() -> std::io::Result<()> {
	#[cfg(windows)]
	winres::WindowsResource::new().set_icon("images/icons/icon.ico").compile()?;
	Ok(())
}
