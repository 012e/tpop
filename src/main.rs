#![allow(dead_code)]
mod session;
mod tmux;
use std::process::exit;

use clap::{Parser, Subcommand};

use crate::session::Session;
#[derive(Parser, Debug)]
struct Args {
	#[clap(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	Toggle,
	Show,
	Hide,
	Create,
	Run {
		#[clap(short, long, default_value_t = false)]
		silent: bool,

		#[arg(required = true)]
		command: String,

		name: Option<String>,
	},
	PrettyRun {
		#[arg(required = true)]
		command: String,
	},
	Digest,
}

fn wrap_command(command: String) -> String {
	format!("bash -c '{}; read'", command)
}

fn main() {
	let args = Args::parse();
	match args {
		Args {
			command: Commands::Toggle,
		} => {
			if let Err(e) = Session::current()
				.expect("must be able to get current session")
				.toggle_popup()
			{
				eprintln!("Coudln't properly create popup: {}", e);
				exit(1);
			}
		}
		Args {
			command: Commands::Create,
		} => {
			if let Err(e) = Session::current()
				.expect("must be able to get current session")
				.ensure_popup_session_exist()
			{
				eprintln!("Couln't prooperly create popup session: {}", e);
				exit(1);
			}
		}
		Args {
			command: Commands::Show,
		} => {
			if let Err(e) = Session::current()
				.expect("must be able to get current session")
				.show_popup()
			{
				eprintln!("Couldn't properly show popup popup: {}", e);
				exit(1);
			}
		}
		Args {
			command: Commands::Hide,
		} => {
			if let Err(e) = Session::current()
				.expect("must be able to get current session")
				.hide_popup()
			{
				eprintln!("Couldn't properly hide popup popup: {}", e);
				exit(1);
			}
		}
		Args {
			command: Commands::Run {
				command,
				silent,
				name,
			},
		} => {
			let session = Session::current().expect("must be able to get current session");
			if let Err(e) = session.spawn_command_window(session::NewCommandPaneConfig {
				command: wrap_command(command),
				path: session.current_path.clone(),
				silent,
				name,
			}) {
				eprintln!("Error while running command: {}", e);
				exit(1);
			}
		}
		Args {
			command: Commands::PrettyRun { command },
		} => {
			println!("pretty run: {}", command);
		}
		Args {
			command: Commands::Digest,
		} => {
			if let Err(e) = Session::current()
				.expect("must be able to get current session")
				.convert_pane_to_popup()
			{
				eprintln!("Couldn't convert pane to popup: {}", e);
				exit(1);
			}
		}
	}
}
