// This [build.rs] is for setting Windows icons.
// The icon in [File Explorer] gets set here.
// The icon in the taskbar and top of the App window gets
// set in [src/main.rs, src/constants.rs] at runtime with
// pre-compiled bytes using [include_bytes!()] on the images in [images/].
#[cfg(windows)]
fn main() -> std::io::Result<()> {
	static_vcruntime::metabuild();
	let mut res = winres::WindowsResource::new();
	// This sets the icon.
	res.set_icon("images/icons/icon.ico");
	// This sets the [Run as Administrator] metadata flag for Windows.
	// Why do I do this?: [https://github.com/hinto-janai/gupax/tree/main/src#why-does-gupax-need-to-be-admin-on-windows]
	// TL;DR: Because Windows.
	res.set_manifest(r#"
	<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
		<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
			<security>
				<requestedPrivileges>
					<requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
				</requestedPrivileges>
			</security>
		</trustInfo>
	</assembly>
	"#);
	res.compile()
}

#[cfg(unix)]
fn main() {}
