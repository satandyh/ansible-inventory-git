use git2::{Cred, FetchOptions, RemoteCallbacks};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Error as IOError;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Deserialize)]
struct Config {
    repo_ssh_address: String,
    key_path: String,
    branch: String,
    target: String,
}

fn read_config(file_path: &str) -> Result<Config, IOError> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents).unwrap();
    Ok(config)
}

fn git_clone(
    repo_ssh_address: &str,
    key_path: &str,
    branch: &str,
    workdir: &str,
) -> Result<(), git2::Error> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(username_from_url.unwrap(), None, Path::new(key_path), None)
    });

    // Prepare fetch options
    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    // Prepare builder
    let mut repo = git2::build::RepoBuilder::new();
    repo.fetch_options(fetch_options);

    // Clone the project
    repo.branch(branch)
        .clone(repo_ssh_address, Path::new(workdir))?;

    Ok(())
}

fn generate_config() {
    println!(
        r#"# repo with inventory file
        repo_ssh_address: ssh://git@github.com:your/inventory/repo.git
        # absolute path to private ssh key (should be accessible)
        key_path: /absolute/path/private_key
        # branch name of repo with inventory file
        branch: any-name
        # relative path to inventory directory or inventory file (inventory.yaml) - it will be used with ansible command
        target: inventory
        "#
    );
}

fn print_help() {
    let app = env::current_exe()
        .expect("Error: Can't get the exec path")
        .file_name()
        .expect("Error: Can't get the exec name")
        .to_string_lossy()
        .into_owned();

    println!(
        "Standalone Usage: {} [-c CONFIG_FILE | --config[=]CONFIG_FILE] [--host HOST] [--list] [-g | --generate-config] [-h | --help]\n",
        app
    );
    println!("Options:");
    println!(
        r#"  -c, --config[=]CONFIG_FILE    Set the absolute path to the config file.
                                By default it use work directory and application filename with ".yaml" at the end.
                                Example, if application filename is "ans-git-inv", then default config filename
                                will be "ans-git-inv.yaml" in the same directory."#
    );
    println!("  --host[=]HOST                 Output specific host info.");
    println!("  --list                        Output all hosts info.");
    println!("  -g, --generate-config         Generate example of config file to stdout.");
    println!("  -h, --help                    Show help.\n");

    println!(
        r#"Usage with Ansible:
  1. Check your ansible.cfg file: script statement should present in enable_plugins option.
  2. Place app and it's config (named the same as app file but with .yaml) somewhere and remember path.
  3. Use next command to check that all works:
    ansible -i /some/folder/ans-inv-git lovely_host -m ping
  4. Use ansible as you always do:
    ansible-playbook -i /some/folder/ans-inv-git --diff plays/lovely_play.yml -l lovely_host"#
    );
}

fn main() {
    // let's use the same full path with name for config, but with ".yaml" at the end
    let full_name = env::current_exe()
        .expect("Error: Can't get the exec path")
        .to_string_lossy()
        .into_owned();

    let mut config_file_path = format!("{}.yaml", full_name);
    // filter for ansible - output specific host info
    let mut host = "".to_string();

    // Random directory to clone repo
    let mut workdir = env::temp_dir();
    let random_path: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    workdir.push(random_path);

    // get vars from arguments
    let args: Vec<String> = env::args().collect();
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            x if (x.starts_with("-c") || x.starts_with("--config")) => {
                if x.starts_with("--config=") {
                    let parts: Vec<&str> = x.split('=').collect();
                    if parts.len() == 2 && parts[1].len() > 1 {
                        config_file_path = parts[1].to_string();
                    } else {
                        eprintln!("Error: Invalid key-value format. Expected: --config=value");
                        print_help();
                        return;
                    }
                } else {
                    if let Some(value) = iter.peek() {
                        if !value.starts_with("-") {
                            config_file_path = value.clone().to_string();
                        } else {
                            eprintln!("Error: Missing config file path");
                            print_help();
                            return;
                        }
                    } else {
                        eprintln!("Error: Missing config file path");
                        print_help();
                        return;
                    }
                }
            }
            x if x.starts_with("--host") => {
                if x.starts_with("--host=") {
                    let parts: Vec<&str> = x.split('=').collect();
                    if parts.len() == 2 && parts[1].len() > 1 {
                        host = parts[1].to_string();
                    } else {
                        eprintln!("Error: Invalid key-value format. Expected: --host=value");
                        print_help();
                        return;
                    }
                } else {
                    if let Some(value) = iter.peek() {
                        if !value.starts_with("-") {
                            host = value.clone().to_string();
                        } else {
                            eprintln!("Error: Missing config file path");
                            print_help();
                            return;
                        }
                    } else {
                        eprintln!("Error: Missing config file path");
                        print_help();
                        return;
                    }
                }
            }
            "--list" => {
                // do nothing
                // because this is exec by default and we not need any special condition here
            }
            "-g" | "--generate-config" => {
                generate_config();
                return;
            }
            "-h" | "--help" => {
                print_help();
                // and exit from process
                return;
            }
            _ => {}
        }
    }

    // read config
    let config = match read_config(&config_file_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error reading configuration file: {}", err);
            return;
        }
    };

    // try to clone repo
    if let Err(err) = git_clone(
        &config.repo_ssh_address,
        &config.key_path,
        &config.branch,
        &workdir.to_string_lossy(),
    ) {
        eprintln!("Error cloning repository: {}", err);
        return;
    }

    // inventory dir or file path - for ansible command
    let target_path = &workdir
        .clone()
        .as_path()
        .join(&config.target)
        .to_string_lossy()
        .to_string();

    // var contains our output from ansible command
    let output: Result<Output, std::io::Error>;
    // if "--host anyhost" present then try to get info about it
    if !host.is_empty() {
        output = Command::new("ansible-inventory")
            .current_dir(&workdir)
            .env("ANSIBLE_INVENTORY_ENABLED", "host_list,auto,yaml,ini,toml")
            .arg("--host")
            .arg(&host)
            .arg("-i")
            .arg(&target_path)
            .output();
    // else just list all hosts
    } else {
        output = Command::new("ansible-inventory")
            .current_dir(&workdir)
            .env("ANSIBLE_INVENTORY_ENABLED", "host_list,auto,yaml,ini,toml")
            .arg("--list")
            .arg("-i")
            .arg(&target_path)
            .output();
    }

    // print to stdout if all alright
    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("{}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Error executing 'ansible-inventory' command: {}", stderr);
            }
        }
        Err(err) => {
            eprintln!("Error executing 'ansible-inventory' command: {}", err);
        }
    }

    // remove temp directory
    if let Err(err) = fs::remove_dir_all(&workdir) {
        if err.kind() != ErrorKind::NotFound {
            eprintln!("Error removing working directory: {}", err);
        }
    }
}
