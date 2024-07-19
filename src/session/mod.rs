pub mod state;
use std::process::Command;

use color_eyre::eyre::Result;

use crate::tmux::{
	self,
	popup::{new_popup, NewPopupConfig},
};

pub struct NewCommandPaneConfig {
	pub command: String,
	pub path: String,
	pub silent: bool,
	pub name: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum PopupEror {
	#[error("Popup already exists")]
	AlreadyExists,
	#[error("Popup does not exist")]
	DoesNotExist,
	#[error("Currently in normal mode, can't hide anything")]
	CantHideNormalMode,
	#[error("Currently in popup mode, can't show anything")]
	CantShowPopup,
}

pub struct Session {
	pub state: state::State,
	pub name: String,
	pub current_path: String,
}

fn wrap_command(command: String) -> String {
	format!("sh -c '{}; read'", command)
}

impl Session {
	pub fn current() -> Result<Self> {
		let state = state::get_state()?;
		let name = tmux::commands::get_session_name()?;
		let current_path = tmux::commands::get_current_session_property("#{pane_current_path}")?;
		Ok(Self {
			state,
			name,
			current_path,
		})
	}

	fn get_popup_session_name(&self) -> String {
		if self.name.starts_with("popup") {
			self.name.clone()
		} else {
			format!("popup{}", self.name)
		}
	}

	pub fn toggle_popup(&self) -> Result<()> {
		match self.state {
			state::State::Normal => self.show_popup()?,
			state::State::Popup => self.hide_popup()?,
		}
		Ok(())
	}

	pub fn hide_popup(&self) -> Result<()> {
		if self.state == state::State::Popup {
			Command::new("tmux").arg("detach").spawn()?;
		}
		Ok(())
	}

	pub fn show_popup(&self) -> Result<()> {
		if let state::State::Normal = self.state {
			self.create_popup_session()?;
			new_popup(NewPopupConfig {
				command: Some("tmux attach -t ".to_owned() + &self.get_popup_session_name()),
				path: self.current_path.clone(),
				height: Some(80),
				width: Some(80),
			})?;
		}
		Ok(())
	}

	pub fn add_popup_pane(&self) -> Result<()> {
		Ok(())
	}

	pub fn spawn_command_window(
		&self,
		NewCommandPaneConfig {
			command,
			path,
			silent,
			name,
		}: NewCommandPaneConfig,
	) -> Result<()> {
		self.create_popup_session()?;
		let mut cmd = Command::new("tmux");
		cmd.arg("new-window")
			.arg("-t")
			.arg(&self.get_popup_session_name())
			.arg("-n")
			.arg(name.unwrap_or("cmdRunner".to_owned()))
			.arg(command)
			.spawn()?;

		if !silent {
			Command::new("tmux")
				.arg("set-hook")
				.arg("-t")
				.arg(&self.get_popup_session_name())
				.arg("pane-exited")
				.arg("detach")
				.spawn()?;
			self.show_popup()?;
		}

		Ok(())
	}

	pub fn create_popup_session(&self) -> Result<()> {
		let popup_session_name = format!("popup{}", self.name);
		Command::new("tmux")
			.arg("new-session")
			.arg("-d")
			.arg("-s")
			.arg(popup_session_name)
			.arg("-c")
			.arg(self.current_path.clone())
			.spawn()?;
		Ok(())
	}
}
