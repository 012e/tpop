use std::process::Command;

use color_eyre::eyre::Result;

pub struct PopupConfig {
	pub command: Option<String>,
	pub path: String,
	pub height: Option<usize>,
	pub width: Option<usize>,
}

pub fn open_popup(
	PopupConfig {
		command,
		path,
		width,
		height,
	}: PopupConfig,
) -> Result<()> {
	let mut cmd = Command::new("tmux");
	cmd.arg("popup").arg("-d").arg(path);

	if let Some(width) = width {
		cmd.arg("-w").arg(width.to_string() + "%");
	}

	if let Some(height) = height {
		cmd.arg("-h").arg(height.to_string() + "%");
	}

	if let Some(command) = command {
		cmd.arg("-E").arg(command);
	}

	cmd.status()?;
	Ok(())
}
