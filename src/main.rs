use chrono::offset::Local;
use dirs::home_dir;
use notify_rust::Notification;
use structopt::StructOpt;
use swayipc::reply::{Node, NodeType};
use wl_clipboard_rs::copy::{self, copy, MimeType};

use std::{
    error::Error,
    io::{Read, Write},
    path::Path,
    path::PathBuf,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

type Geometry = String;

fn node_geometry(node: &Node) -> Geometry {
    let rect = &node.rect;
    format!("{},{} {}x{}", rect.x, rect.y, rect.width, rect.height)
}

fn visible_workspaces() -> Result<Vec<Node>> {
    let mut sway = swayipc::Connection::new()?;

    let root = sway.get_tree()?;

    let outputs = root
        .nodes
        .into_iter()
        .filter(|n| n.node_type == NodeType::Output);

    let workspaces = outputs
        .into_iter()
        .flat_map(|o| o.nodes)
        .filter(|n| n.node_type == NodeType::Workspace);

    let visible_workspace_names = sway
        .get_workspaces()?
        .into_iter()
        .filter(|w| w.visible)
        .map(|w| w.name)
        .collect::<Vec<_>>();

    let visible_workspaces = workspaces.filter(|w| {
        let name = w.name.clone().expect("Workspace should have a name.");
        visible_workspace_names.contains(&name)
    });

    Ok(visible_workspaces.collect())
}

fn visible_windows() -> Result<Vec<Geometry>> {
    let mut node_queue = visible_workspaces()?;
    let mut output = Vec::new();

    while let Some(node) = node_queue.pop() {
        if node.pid.is_some() {
            output.push(node_geometry(&node));
        }

        node_queue.extend(node.nodes);
        node_queue.extend(node.floating_nodes);
    }

    Ok(output)
}

fn select_region() -> Result<Option<Geometry>> {
    let mut process = Command::new("slurp")
        .args(&["-b", "282a3666"])
        .args(&["-c", "ff79c6"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let input = visible_windows()?.join("\n");

    process
        .stdin
        .as_mut()
        .map(|stdin| stdin.write_all(input.as_bytes()))
        .unwrap()?;

    process.wait()?;

    let mut output = String::new();
    let mut stdout = process.stdout.unwrap();
    stdout.read_to_string(&mut output)?;

    Ok(if output.is_empty() {
        None
    } else {
        Some(output.trim().to_owned())
    })
}

fn notify(message: &str) -> Result<()> {
    Notification::new()
        .summary("screenshot")
        .timeout(2000)
        .body(message)
        .show()?;

    Ok(())
}

enum Output {
    File(PathBuf),
    Clipboard,
}

fn take_screenshot(region: Option<Geometry>, output: Output) -> Result<()> {
    let mut process = Command::new("grim");

    if let Some(geometry) = region {
        process.args(&["-g", &geometry]);
    }

    match output {
        Output::File(path) => {
            process.arg(&path).output()?;
            notify(&format!("saved to {}", path.display()))
        }

        Output::Clipboard => {
            let output = process.arg("-").output()?;
            copy(
                copy::Options::new(),
                copy::Source::Bytes(&output.stdout),
                MimeType::Specific("image/png".to_owned()),
            )
            .map_err(|e| format!("{}", e))?;

            notify("copied to clipboard")
        }
    }
}

fn start_recording(
    region: Option<String>,
    path: &Path,
    audio: bool,
) -> Result<std::process::Child> {
    let mut process = Command::new("wf-recorder");

    if let Some(geometry) = region {
        process.args(&["-g", &geometry]);
    }

    process.arg("-f").arg(path);

    if audio {
        process.arg("-a0");
    }

    let message = format!("starting recording to be saved at {}", path.display());
    notify(&message)?;

    sleep(Duration::from_secs(2));

    Ok(process.spawn()?)
}

#[derive(StructOpt)]
/// Screen capture assistant for Wayland.
struct Args {
    #[structopt(subcommand)]
    subcommand: Subcommand,

    /// Selects a region or window to be captured
    #[structopt(short, long)]
    selection: bool,
}

#[derive(StructOpt, PartialEq)]
enum Subcommand {
    /// Takes a screenshot
    Screenshot {
        #[structopt(short, long)]
        /// Copies to clipboard instead of saving to a file
        copy: bool,
    },

    /// Starts a video recording
    Record {
        #[structopt(short, long)]
        /// Captures audio when recording video
        audio: bool,
    },

    /// Stops recording by killing all wf-recorder processes
    Kill,
}

fn output_path(extension: &str) -> PathBuf {
    home_dir()
        .expect("Couldn't detect home directory.")
        .join(Local::now().to_rfc3339())
        .with_extension(extension)
}

fn main() -> Result<()> {
    let args = Args::from_args();

    if args.subcommand == Subcommand::Kill {
        Command::new("killall")
            .arg("-s2")
            .arg("wf-recorder")
            .output()?;

        return notify("stopped recording");
    }

    let region = if args.selection {
        match select_region()? {
            None => return notify("cancelled selection"),
            r => r,
        }
    } else {
        None
    };

    match args.subcommand {
        Subcommand::Record { audio } => {
            start_recording(region, &output_path("mp4"), audio)?;
        }
        Subcommand::Screenshot { copy } => {
            let output = if copy {
                Output::Clipboard
            } else {
                Output::File(output_path("png"))
            };
            take_screenshot(region, output)?;
        }
        _ => {}
    }

    Ok(())
}
