use core::panic;
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
struct ComfyData {
    command: String,
    flag: String,
    comfypath: String,
    params: String,
}
fn listdir() -> Result<Vec<String>, &'static str> {
    let dir = fs::read_dir("C:\\ComfyPSD-backend\\custom_nodes");
    let mut result = Vec::new();
    match dir {
        Ok(dirs) => {
            for entry in dirs {
                if let Ok(entry) = entry {
                    let metadata = entry.metadata().unwrap();
                    if let Some(fname) = entry.file_name().to_str() {
                        if metadata.is_dir() && fname != "__pycache__" {
                            result.push(String::from(fname));
                        }
                    }
                }
            }
        }
        Err(_) => panic!("hello"),
    }
    Ok(result)
}
fn run_update(menu: &Vec<String>, selection: usize, data: &ComfyData) {
    let command = format!(
        "{} cd {}custom_nodes\\{} && git pull",
        data.flag, data.comfypath, menu[selection]
    );

    let cmd = Command::new(&data.command)
        .args(command.split_whitespace())
        .stdout(Stdio::piped())
        .spawn()
        .expect("error doing things");
    if let Some(stdout) = cmd.stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{:?}", line.unwrap());
        }
    }

    list_all_nodes(&menu, data);
}
fn list_all_nodes(menu: &Vec<String>, data: &ComfyData) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("what to do:")
        .default(0)
        .items(&menu[..])
        .interact()
        .unwrap();
    if selection == &menu.len() - 1 {
        main();
    } else {
        run_update(&menu, selection, data);
    }
}
fn run_server(data: &ComfyData) {
    let command = format!("{} cd {} {}", &data.flag, &data.comfypath, &data.params);

    let cmd = Command::new(&data.command)
        .args(command.split_whitespace())
        .stdout(Stdio::piped())
        .spawn()
        .expect("error doing things");
    if let Some(stdout) = cmd.stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line.unwrap())
        }
    }
}
fn install_customnodes(data: &ComfyData) {
    let git_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("git url")
        .default("http://github.com/johndoe/awesomenode.git".to_string())
        .interact_text()
        .unwrap();
    if git_url == "http://github.com/johndoe/awesomenode.git" {
        main();
        return;
    }
    let command = format!(
        "{} cd {}custom_nodes && git clone {}",
        data.flag, data.comfypath, git_url
    );

    let cmd = Command::new(&data.command)
        .args(command.split_whitespace())
        .stdout(Stdio::piped())
        .spawn()
        .expect("error doing things");
    if let Some(stdout) = cmd.stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line.unwrap())
        }
    }
}

fn edit_config(data: &ComfyData) {
    let command_to_use: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("command (ie. pwsh,cmd,sh)")
        .default(String::from(&data.command))
        .interact_text()
        .unwrap();
    let command_flag: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("flag (-c or /C)")
        .default(String::from(&data.flag))
        .interact_text()
        .unwrap();
    let comfy_path: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("comfyui-path")
        .default(String::from(&data.comfypath))
        .interact_text()
        .unwrap();
    let arguments: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("comfyui-arguments")
        .default(String::from(&data.params))
        .interact_text()
        .unwrap();
    let default_data = ComfyData {
        command: command_to_use,
        flag: command_flag,
        comfypath: comfy_path,
        params: arguments,
    };
    let file_name = get_config_path();
    let serialized = serde_json::to_string_pretty(&default_data).unwrap();
    let mut file = File::create(&file_name).unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
    main();
}
fn get_config_path() -> String {
    let configfile = PathBuf::from(std::env::current_exe().unwrap())
        .parent()
        .unwrap()
        .join("comfy_config.json");

    match configfile.exists() {
        true => configfile.to_string_lossy().to_string(),
        false => {
            let file_name = configfile.to_string_lossy().to_string();
            let default_data = ComfyData {
                command:"pwsh".into(),
                flag:"-c".into(),
                comfypath: "C:/ComfyPSD-backend/".into(),
                params: "&& conda activate comfyui && python main.py --lowvram --dont-upcast-attention --preview-method auto".into(),
            };
            let serialized = serde_json::to_string_pretty(&default_data).unwrap();
            let mut file = File::create(&file_name).unwrap();
            file.write_all(serialized.as_bytes()).unwrap();
            file_name
        }
    }
}
fn read_config() -> Result<ComfyData, serde_json::Error> {
    let file_name = get_config_path();
    let data_result = match fs::read_to_string(&file_name) {
        Ok(content) => serde_json::from_str::<ComfyData>(&content),
        Err(_) => panic!("cant write shit"),
    };
    data_result
}

fn delete_input_output_folder(data: &ComfyData) {
    let menu = &["input", "output"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("delete folder content of")
        .default(0)
        .items(&menu[..])
        .interact()
        .unwrap();

    let dir_content = fs::read_dir(
        PathBuf::from(&data.comfypath)
            .join(&menu[selection])
            .to_string_lossy()
            .to_string(),
    );
    match dir_content {
        Ok(dir_entries) => {
            for entry in dir_entries {
                if let Ok(entry) = entry {
                    if let Some(file_path) = entry.path().to_str() {
                        if entry.metadata().unwrap().is_file() {
                            match fs::remove_file(file_path) {
                                Ok(_) => {
                                    println!("deleted:{}", entry.file_name().to_str().unwrap())
                                }
                                Err(_) => println!(
                                    "failed to remove {}",
                                    entry.file_name().to_str().unwrap()
                                ),
                            }
                        }
                    }
                }
            }
        }
        Err(_) => main(),
    }
}

fn update_comfyui(data: &ComfyData) {
    let command = format!("{} cd {} && git pull", data.flag, data.comfypath);
    let cmd = Command::new(&data.command)
        .args(command.split_whitespace())
        .stdout(Stdio::piped())
        .spawn()
        .expect("error doing things");
    if let Some(stdout) = cmd.stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{:?}", line.unwrap());
        }
    }
    thread::sleep(Duration::from_secs(1));
    main();
}

fn main_menu(menu: &Vec<&str>, items: Vec<String>, data: ComfyData) {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(&menu[..])
        .interact()
        .unwrap();

    match selection {
        0 => edit_config(&data),
        1 => run_server(&data),
        2 => update_comfyui(&data),
        3 => install_customnodes(&data),
        4 => list_all_nodes(&items, &data),
        5 => delete_input_output_folder(&data),
        _ => std::process::exit(0),
    }
}
fn main() {
    print!("\x1B[2J\x1B[1;1H");
    let cdata = read_config();

    match cdata {
        Ok(data) => {
            println!(
                r#"
██████╗ ██████╗  ███╗   ███╗███████╗██╗   ██╗██╗   ██╗██╗
██╔════╝██╔═══██╗████╗ ████║██╔════╝╚██╗ ██╔╝██║   ██║██║
██║     ██║   ██║██╔████╔██║█████╗   ╚████╔╝ ██║   ██║██║
██║     ██║   ██║██║╚██╔╝██║██╔══╝    ╚██╔╝  ██║   ██║██║
╚██████╗╚██████╔╝██║ ╚═╝ ██║██║        ██║   ╚██████╔╝██║
 ╚═════╝ ╚═════╝ ╚═╝     ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝
      
---command             :{}
---flag                :{}
---comfyui path        :{}
---arguments           :{}

"#,
                data.command, data.flag, data.comfypath, data.params
            );

            let menu = vec![
                "edit config (!important)",
                "run server",
                "update comfyui",
                "install custom nodes (git)",
                "update nodes (git)",
                "delete input/output",
                "quit",
            ];
            let mut items = listdir().unwrap();
            items.push("quit".into());
            main_menu(&menu, items, data);
        }
        Err(_) => panic!("cant read that shit!!!"),
    }
}
