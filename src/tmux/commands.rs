use std::{
	io,
	process::{Command, ExitStatus, Stdio},
};

pub fn new_session_detached(name: &str, create_new: bool) -> Result<(), io::Error> {
	let mut cmd = Command::new("tmux");
	cmd.arg("new-session").arg("-d").arg("-s").arg(name);
	if !create_new {
		cmd.arg("-A");
	}
	cmd.status()?;
	Ok(())
}

pub fn new_session(name: &str, create_new: bool) -> Result<(), io::Error> {
	let mut cmd = Command::new("tmux");
	cmd.arg("new-session").arg("-s").arg(name);
	if !create_new {
		cmd.arg("-A");
	}
	cmd.status()?;
	Ok(())
}

pub fn run_command(command: &str, target: &str) -> Result<(), io::Error> {
	Command::new("tmux")
		.arg("send-keys")
		.arg("-t")
		.arg(target)
		.arg(command)
		.arg("Enter")
		.status()?;
	Ok(())
}

pub fn get_current_session_property(property: &str) -> Result<String, io::Error> {
	let output = Command::new("tmux")
		.arg("display-message")
		.arg("-p")
		.arg(property)
		.output()?;
	Ok(String::from_utf8(output.stdout).unwrap().trim().into())
}

pub fn get_session_property(property: &str, target: &str) -> Result<String, io::Error> {
	let output = Command::new("tmux")
		.arg("display-message")
		.arg("-p")
		.arg(property)
		.arg("-t")
		.arg(target)
		.output()?;
	Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn get_session_name() -> Result<String, io::Error> {
	get_current_session_property("#{session_name}")
}

pub fn move_window(
	src_session: String,
	window_index: String,
	dest_session: String,
) -> Result<(), io::Error> {
	let src = format!("{src_session}:{window_index}");
	Command::new("tmux")
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.arg("move-window")
		.arg("-s")
		.arg(src)
		.arg("-t")
		.arg(dest_session)
		.status()?;
	Ok(())
}

pub fn has_session(s: String) -> Result<bool, io::Error> {
	let exit_status = Command::new("tmux")
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.arg("has-session")
		.arg("-t")
		.arg(s)
		.status()?;
	Ok(exit_status.success())
}
