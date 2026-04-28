use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, Write},
    // path::Path,
};

const CONFIG_FILE_PATH: &str = ".pxsol-ss-client/config.json";

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config {
            program_id,
            network,
            get,
        } => config_set(program_id, network)?,
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "pxsol-ss-client")]
#[command(version = "0.0.1")]
#[command(about = "Test For pxsol-ss",long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Config {
        #[arg(short = 'p', long = "programid")]
        program_id: Option<String>,

        #[arg(short = 'n', long = "network")]
        network: Option<String>,

        #[arg(short = 'g', long = "get", default_value_t = false)]
        get: bool,
    },
}

pub fn config_set(program_id: Option<String>, network: Option<String>) -> Result<()> {
    // 文件路径拼接
    let home_path = dirs::home_dir().expect("无法获取 home 目录");
    let path = home_path.join(CONFIG_FILE_PATH);

    println!(
        "新传入配置为 program_id:{:?} , network: {:?}",
        program_id, network
    );

    if !path.exists() {
        //文件不存在创建文件
        println!("配置文件不存在！开始创建...");

        // 递归创建文件
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if path.exists() {
            println!("创建成功");
        } else {
            println!("创建失败");
        }
    } else {
        //文件已存在
        let file = File::open(path.clone())?;
        let reader = BufReader::new(file);
        let config_json: ConfigJson = serde_json::from_reader(reader)?;
        println!(
            "配置文件已存在 当前配置为 program_id:{} , network: {} ",
            config_json.program_id, config_json.network
        );
    }

    // 配置写入
    let mut config_json = ConfigJson::new();
    if !program_id.is_none() {
        config_json.program_id = program_id.clone().unwrap();
    }
    if !network.is_none() {
        config_json.network = network.clone().unwrap();
    }
    let json_string = serde_json::to_string_pretty(&config_json)?;
    let mut file = File::create(&path)?;
    file.write_all(json_string.as_bytes())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ConfigJson {
    program_id: String,
    network: String,
}

impl ConfigJson {
    fn new() -> ConfigJson {
        ConfigJson {
            program_id: String::new(),
            network: String::new(),
        }
    }

    fn get(self)->ConfigJson{
        ConfigJson {
            program_id: self.program_id,
            network: self.network,
        }
    }
}
