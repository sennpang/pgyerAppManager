use std::{collections::HashMap, env, fs, path::Path, process, thread, time::Duration};

use clap::{App, Arg, ArgMatches};
use reqwest::multipart::{self, Form};
use serde_json::{Result, Value};
const INSTALL_PASSWORD: &str = "2";
async fn upload_file(data: &Value) -> Result<()> {
    let matches = get_command_params();

    let file_path = matches.value_of("file");
    let file_name = Path::new(file_path.unwrap()).file_name();
    let form = reqwest::multipart::Form::new()
        .text(
            "signature",
            data["params"]["signature"].as_str().unwrap().to_owned(),
        )
        .text(
            "x-cos-security-token",
            data["params"]["x-cos-security-token"]
                .as_str()
                .unwrap()
                .to_owned(),
        )
        .text("key", data["key"].as_str().unwrap().to_owned())
        // .part("file", Part::stream(body));
        .part(
            "file",
            reqwest::multipart::Part::bytes(std::fs::read(&file_path.unwrap()).expect("233333"))
                .file_name(file_name.unwrap().to_string_lossy().to_string()),
        );

    let client = reqwest::Client::builder().build().unwrap();
    let request = client
        .request(
            reqwest::Method::POST,
            data["endpoint"].as_str().unwrap().to_owned(),
        )
        .multipart(form);

    println!("上传中...");
    request.send().await.expect("上传失败!");
    // let body = response.text().await.unwrap();
    let mut build_code = 1246;
    let build_deal_code: Vec<i32> = vec![1246, 1247];
    let error_code = 1216;

    println!("上传完成, 服务端处理中...");
    while build_code >= 0 {
        let build_info = get_build_info(data["key"].as_str().unwrap().to_string())
            .await
            .unwrap();
        build_code = build_info.get("code").unwrap().as_i64().unwrap() as i32;
        thread::sleep(Duration::from_secs(1));

        if build_deal_code.contains(&build_code) {
            continue;
        }

        if build_code == 0 {
            println!(
                "buildVersion: {}",
                build_info["data"]["buildBuildVersion"]
                    .as_str()
                    .unwrap()
                    .to_string()
            );
            println!(
                "buildCreated: {}",
                build_info["data"]["buildCreated"]
                    .as_str()
                    .unwrap()
                    .to_string()
            );
            println!(
                "buildDescription: {}",
                build_info["data"]["buildDescription"]
                    .as_str()
                    .unwrap()
                    .to_string()
            );
            println!(
                "buildQRCodeURL: {}",
                build_info["data"]["buildQRCodeURL"]
                    .as_str()
                    .unwrap()
                    .to_string()
            );
            println!(
                "buildShortcutUrl: https://www.pgyer.com/{}",
                build_info["data"]["buildShortcutUrl"]
                    .as_str()
                    .unwrap()
                    .to_string()
            );
            process::exit(0);
        }

        if build_code == error_code {
            println!("服务端处理失败了!");
            break;
        }

        println!("{}", build_info);
        process::exit(1);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    check_params();
    let matches = get_command_params();
    if matches.value_of("file").is_some() {
        upload().await;
    }

    if matches.value_of("delete").is_some() {
        delete_app(&matches).await;
    }
}

fn check_params() {
    let matches = get_command_params();
    if let Some(api_key) = matches.value_of("api_key") {
        set_api_key(api_key);
        println!("api_key: {}", api_key);
    }

    if let Some(output_file) = matches.value_of("file") {
        println!("file: {}", output_file);
    }

    let api_key = get_api_key();
    if api_key.is_empty() {
        println!("请先设置 api_key");
        process::exit(0);
    }
}

async fn upload() {
    let matches = get_command_params();

    let file_path = matches.value_of("file");
    let mut build_type = "apk";
    if let Some(name) = file_path {
        let name_str = name.to_string();
        if fs::metadata(&name_str).is_err() {
            println!("文件不存在!");
            process::exit(0);
        }

        let extension = Path::new(name).extension().and_then(|ext| ext.to_str());

        let build_deal_code = vec!["apk", "ipa"];
        match extension {
            Some(ext) => {
                build_type = ext;
                if !build_deal_code.contains(&ext) {
                    println!("只支持ipa/apk");
                    process::exit(0);
                }
            }
            None => println!("文件格式不正确"),
        }
    } else {
        println!("请携带文件参数来上传应用, -h 获取更多帮助");
        process::exit(0);
    }

    let token_info = get_cos_token(&matches, &build_type.to_string())
        .await
        .unwrap();
    let res = upload_file(&token_info.get("data").unwrap()).await;
    match res {
        error => println!("{:?}", error),
    }
}

fn get_command_params() -> ArgMatches<'static> {
    let matches = App::new("PGYER APP MANAGER")
        .version("0.1")
        .author("PANG")
        .about("PGYER APP MANAGER")
        .arg(
            Arg::with_name("api_key")
                .short("k")
                .long("key")
                .value_name("STRING")
                .help("Sets the api key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("delete")
                .short("d")
                .long("delete")
                .value_name("STRING")
                .help("app key that you want to delete")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Sets the upload file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("channel")
                .short("c")
                .long("channel")
                .value_name("STRING")
                .help("build channel shortcut")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("installEndDate")
                .short("e")
                .long("installEndDate")
                .value_name("STRING")
                .help("build install start date, format: yyyy-MM-dd")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("installStartDate")
                .short("s")
                .long("installStartDate")
                .value_name("STRING")
                .help("build install end date, format: yyyy-MM-dd")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("installDate")
                .short("i")
                .long("installDate")
                .value_name("NUMBER")
                .help("build install date, 1=buildInstallStartDate~buildInstallEndDate, 2=forever")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("description")
                .short("d")
                .long("description")
                .value_name("STRING")
                .help("build update description")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .value_name("STRING")
                .help("build password, required if installType=2")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("installType")
                .short("t")
                .long("installType")
                .value_name("NUMBER")
                .help("build install type, 1=public, 2=password, 3=invite")
                .takes_value(true),
        )
        .get_matches();
    if matches.args.is_empty() {
        println!("-h 获取帮助信息");
        process::exit(0);
    }

    matches
}

fn set_api_key(api_key: &str) {
    if api_key.len() != 32 {
        println!("api_key is invalid");
        process::exit(0);
    }

    let db: sled::Db = sled::open("my_db").unwrap();
    db.insert("_api_key", api_key).unwrap();
}

fn get_api_key() -> String {
    let db: sled::Db = sled::open("my_db").unwrap();
    if let Ok(Some(value)) = db.get("_api_key") {
        String::from_utf8(value.to_vec()).expect("_api_key not init")
    } else {
        if let Ok(pgyer_api_key) = env::var("PGYER_API_KEY") {
            return pgyer_api_key;
        }
        return String::from("");
    }
}

async fn get_cos_token(matches: &ArgMatches<'_>, build_type: &String) -> Result<Value> {
    let api_key = get_api_key();
    let pairs = vec![
        ("_api_key", api_key),
        ("buildType", build_type.to_string()),
        (
            "buildChannelShortcut",
            matches.value_of("channel").unwrap_or("").to_string(),
        ),
        (
            "buildInstallEndDate",
            matches.value_of("installEndDate").unwrap_or("").to_string(),
        ),
        (
            "buildInstallStartDate",
            matches
                .value_of("installStartDate")
                .unwrap_or("")
                .to_string(),
        ),
        (
            "buildInstallDate",
            matches.value_of("installDate").unwrap_or("").to_string(),
        ),
        (
            "buildDescription",
            matches.value_of("description").unwrap_or("").to_string(),
        ),
        (
            "buildPassword",
            matches.value_of("password").unwrap_or("").to_string(),
        ),
        (
            "buildInstallType",
            matches.value_of("installType").unwrap_or("1").to_string(),
        ),
    ];

    if matches.value_of("installType") == Some("2") && matches.value_of("password").is_none() {
        println!("密码安装方式需要传递 password 参数");
        process::exit(0);
    }

    if matches.value_of("installDate") == Some("1")
        && (matches.value_of("installStartDate").is_none()
            || matches.value_of("installEndDate").is_none())
    {
        println!("需要传递安装时间参数");
        process::exit(0);
    }

    let install_end_date = matches.value_of("installEndDate").unwrap_or("");
    let install_start_date = matches.value_of("installStartDate").unwrap_or("");
    if (install_start_date.len() > 0 && install_start_date.len() != 10)
        || (install_end_date.len() > 0 && install_end_date.len() != 10)
    {
        println!("时间参数不正确, 正确格式 yy-MM-DD (2001-02-01)");
        process::exit(0);
    }

    let url = "https://www.pgyer.com/apiv2/app/getCOSToken";
    let res = request(pairs, url.to_owned()).await.unwrap();

    return Ok(res);
}

async fn delete_app(matches: &ArgMatches<'_>) {
    let api_key = get_api_key();
    if matches.value_of("delete").is_none() {
        println!("需要 appKey 参数");
        process::exit(0);
    }

    let pairs = vec![
        ("_api_key", api_key),
        (
            "appKey",
            matches.value_of("delete").unwrap_or("").to_string(),
        ),
    ];

    let url = "https://www.pgyer.com/apiv2/app/deleteApp";
    let res = request(pairs, url.to_owned()).await.unwrap();
    let build_code = res.get("code").unwrap().as_i64().unwrap() as i32;
    println!("删除中...");
    if build_code != 0 {
        println!("{}", res.get("message").unwrap());
        process::exit(0);
    }

    println!("删除成功");
    process::exit(0);
}

async fn get_build_info(build_key: String) -> Result<Value> {
    let api_key = get_api_key();
    let pairs = vec![
        ("_api_key", api_key.clone()),
        ("buildKey", build_key.clone()),
    ];
    let url = format!(
        "https://www.pgyer.com/apiv2/app/buildInfo?_api_key={}&buildKey={}",
        api_key.clone(),
        build_key.clone()
    );
    let res = request(pairs, url.to_owned()).await.unwrap();
    return Ok(res);
}

fn create_form(form_fields: HashMap<&str, String>) -> Form {
    let mut multipart_form = multipart::Form::new();
    for (key, value) in form_fields {
        multipart_form = multipart_form.text(key.to_string(), value.to_string());
    }
    multipart_form
}

async fn request(pairs: Vec<(&str, String)>, url: String) -> Result<Value> {
    // Create a HashMap from the predefined pairs
    let form_fields: HashMap<_, _> = pairs.into_iter().collect();

    let form = create_form(form_fields);
    let client = reqwest::Client::builder().build().unwrap();
    let request = client.request(reqwest::Method::POST, url).multipart(form);

    let response = request.send().await.unwrap();
    let body = response.text().await.unwrap();
    let person: Value = serde_json::from_str(body.as_str()).unwrap();
    Ok(person)
}
