pub mod app {
    use std::{
        collections::HashMap,
        env, fs,
        path::Path,
        process, thread,
        time::{Duration, Instant},
    };

    use chrono::Local;
    use clap::{App, AppSettings, Arg, ArgMatches};
    use reqwest::{
        multipart::{self, Form},
        Client,
    };
    use serde_json::{Result, Value};
    const MB: u64 = 1024 * 1024;
    const GB: u64 = MB * 1024;
    const INSTALL_PASSWORD: &str = "2";
    const INSTALL_AT_DATE_RANGE: &str = "1";
    const PGYER_API_ENDPOINT: &str = "https://www.pgyer.com/apiv2/app/";

    async fn upload_file(data: &Value) -> Result<()> {
        let matches = get_command_params();

        let file_path = matches.value_of("file");
        let file_name = Path::new(file_path.unwrap()).file_name();
        // Retrieve the metadata of the file
        let metadata = fs::metadata(file_path.unwrap()).expect("Failed to read metadata");

        // Extract the file size from the metadata
        let file_size = metadata.len();

        // 大于 2GB 不让传
        if file_size > (2 * GB) {
            println!("当前文件大于 2GB, 无法上传");
            process::exit(0);
        }

        // Start measuring the run time
        let start_time = Instant::now();

        let current_time = Local::now();
        println!("当前时间: {}", current_time);

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
            .part(
                "file",
                reqwest::multipart::Part::bytes(
                    std::fs::read(&file_path.unwrap()).expect("读取文件失败!"),
                )
                .file_name(file_name.unwrap().to_string_lossy().to_string()),
            );

        let client = reqwest::Client::builder().build().unwrap();
        let request = client
            .request(reqwest::Method::POST, data["endpoint"].as_str().unwrap())
            .multipart(form);

        println!("上传中...");

        let response = request.send().await.unwrap();
        let status = response.status();

        if status != 204 {
            println!("上传失败!!!");
            println!("{}", response.text().await.unwrap());
            process::exit(0);
        }

        let mut build_code = 1246;
        let build_deal_code = vec![1246, 1247];
        let error_code = 1216;

        let duration = start_time.elapsed().as_secs_f32();
        println!("上传耗时: {:.2} 秒", duration); // Calculate the run time duration

        println!("上传完成, 服务端处理中...");
        let current_time = Local::now();
        println!("当前时间: {}", current_time);
        while build_code != 0 {
            let build_info = get_build_info(data["key"].as_str().unwrap()).await;
            build_code = build_info.get("code").unwrap().as_i64().unwrap();
            thread::sleep(Duration::from_secs(1));

            if build_deal_code.contains(&build_code) {
                continue;
            }

            if build_code == 0 {
                println!("应用信息: ");
                println!(
                    "buildVersion: {}",
                    build_info["data"]["buildBuildVersion"].as_str().unwrap()
                );
                println!(
                    "buildCreated: {}",
                    build_info["data"]["buildCreated"].as_str().unwrap()
                );
                println!(
                    "buildDescription: {}",
                    build_info["data"]["buildDescription"].as_str().unwrap()
                );
                println!(
                    "buildQRCodeURL: {}",
                    build_info["data"]["buildQRCodeURL"].as_str().unwrap()
                );
                println!(
                    "buildShortcutUrl: https://www.pgyer.com/{}",
                    build_info["data"]["buildShortcutUrl"].as_str().unwrap()
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

    pub fn check_params() {
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

    pub async fn upload() {
        let matches = get_command_params();

        let file_path = matches.value_of("file");
        let build_type;
        if let Some(name) = file_path {
            let name_str = name;
            if fs::metadata(&name_str).is_err() {
                println!("文件不存在!");
                process::exit(0);
            }

            let extension = Path::new(name).extension().and_then(|ext| ext.to_str());

            let build_deal_code = vec!["apk", "ipa"];
            match extension {
                Some(ext) => {
                    build_type = ext;
                    if !build_deal_code.contains(&ext.to_lowercase().as_str()) {
                        println!("只支持ipa/apk");
                        process::exit(0);
                    }
                }
                None => {
                    println!("文件格式不正确");
                    process::exit(0);
                }
            }
        } else {
            println!("请携带文件参数来上传应用, -h 获取更多帮助");
            process::exit(0);
        }

        check_proxy().await;

        let token_info = get_cos_token(&matches, build_type).await.unwrap();
        let res = upload_file(&token_info.get("data").unwrap()).await;
        match res {
            error => println!("{:?}", error),
        }
    }

    pub async fn check_proxy() {
        let client = Client::new();
        let response = client.get("https://www.google.com").send().await;
        match response {
            Ok(res) => {
                // Check the response status
                if res.status().is_success() {
                    // Request was successful, process the response body
                    // let body = res.text().await.unwrap();
                    // println!("Response body: {}", body);
                    println!(
                        "您使用了代理, 可能导致上传失败, 请关闭代理或者使 pgyer.com 走直连通道!"
                    );
                } else {
                    // Request failed with a non-success status code
                    println!("Request failed with status code: {}", res.status());
                }
            }
            Err(_err) => {
                println!("检测网络正常!");
                // Request failed, handle the error
                // println!("Request error: {}", err);
            }
        }

        let response = client
            .get("https://pgy-apps-1251724549.cos.ap-guangzhou.myqcloud.com/")
            .send()
            .await;
        match response {
            Ok(res) => {
                if res.status().is_success() {
                } else if res.status() == 403 {
                    // println!("检测网络正常!");
                } else {
                    println!(
                        "myqcloud.com Request failed with status code: {}",
                        res.status()
                    );
                }
            }
            Err(_err) => {
                println!("Request error: {}", _err);
            }
        }
    }

    fn get_app_cli() -> App<'static, 'static> {
        return App::new("PGYER APP MANAGER")
        .version("0.1")
        .author("PANG")
        .about("PGYER APP MANAGER")
        .arg(
            Arg::with_name("channel")
                .short("c")
                .long("channel")
                .value_name("STRING")
                .help("build channel shortcut")
                .takes_value(true),
        )
        .arg(Arg::with_name("check").long("check").help("check network"))
        .arg(
            Arg::with_name("description")
                .short("d")
                .long("description")
                .value_name("STRING")
                .help("build update description")
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
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Sets the upload file")
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
            Arg::with_name("api_key")
                .short("k")
                .long("key")
                .value_name("STRING")
                .help("Sets the api key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("list my apps"),
        )
        .arg(
            Arg::with_name("page")
                .long("page")
                .value_name("NUMBER")
                .help("page number")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("appKey")
                .short("r")
                .long("remove")
                .value_name("STRING")
                .help("app key that you want to delete")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("buildKey")
                .long("removeBuild")
                .value_name("STRING")
                .help("build key that you want to delete")
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
        .arg(
          Arg::with_name("info")
              .long("info")
              .value_name("STRING")
              .help("get build info with build key")
              .takes_value(true),
      ).setting(AppSettings::ArgRequiredElseHelp);
    }

    pub fn get_command_params() -> ArgMatches<'static> {
        let app = get_app_cli();
        return app.get_matches();
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

    pub async fn get_cos_token(matches: &ArgMatches<'_>, build_type: &str) -> Result<Value> {
        let api_key = get_api_key();
        let pairs: Vec<(&str, &str)> = vec![
            ("_api_key", &api_key),
            ("buildType", build_type),
            (
                "buildChannelShortcut",
                matches.value_of("channel").unwrap_or(""),
            ),
            (
                "buildInstallEndDate",
                matches.value_of("installEndDate").unwrap_or(""),
            ),
            (
                "buildInstallStartDate",
                matches.value_of("installStartDate").unwrap_or(""),
            ),
            (
                "buildInstallDate",
                matches.value_of("installDate").unwrap_or(""),
            ),
            (
                "buildDescription",
                matches.value_of("description").unwrap_or(""),
            ),
            ("buildPassword", matches.value_of("password").unwrap_or("")),
            (
                "buildInstallType",
                matches.value_of("installType").unwrap_or(""),
            ),
        ];

        if matches.value_of("installType") == Some(INSTALL_PASSWORD)
            && matches.value_of("password").is_none()
        {
            println!("密码安装方式需要传递 password 参数");
            process::exit(0);
        }

        if matches.value_of("installDate") == Some(INSTALL_AT_DATE_RANGE)
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

        let url = PGYER_API_ENDPOINT.to_owned() + "getCOSToken";
        let res = request(pairs, &url).await.unwrap();

        return Ok(res);
    }

    pub async fn delete_app(matches: &ArgMatches<'_>) {
        let api_key = get_api_key();
        if matches.value_of("appKey").is_none() {
            println!("需要 appKey 参数");
            process::exit(0);
        }

        let pairs: Vec<(&str, &str)> = vec![
            ("_api_key", &api_key),
            ("appKey", matches.value_of("appKey").unwrap_or("")),
        ];

        println!("删除中...");
        let url = PGYER_API_ENDPOINT.to_owned() + "deleteApp";
        let res = request(pairs, &url).await.unwrap();
        let build_code = res.get("code").unwrap();
        if build_code != 0 {
            println!("{}", res.get("message").unwrap());
            process::exit(0);
        }

        println!("删除成功");
        process::exit(0);
    }

    pub async fn delete_build(matches: &ArgMatches<'_>) {
        let api_key = get_api_key();
        if matches.value_of("buildKey").is_none() {
            println!("需要 buildKey 参数");
            process::exit(0);
        }

        let pairs: Vec<(&str, &str)> = vec![
            ("_api_key", &api_key),
            ("buildKey", matches.value_of("buildKey").unwrap_or("")),
        ];

        println!("删除中...");
        let url = PGYER_API_ENDPOINT.to_owned() + "buildDelete";
        let res = request(pairs, &url).await.unwrap();
        let build_code = res.get("code").unwrap();
        if build_code != 0 {
            println!("{}", res.get("message").unwrap());
            process::exit(0);
        }

        println!("删除成功");
        process::exit(0);
    }

    pub async fn get_app_list(page: &str) {
        let api_key = get_api_key();

        let pairs: Vec<(&str, &str)> = vec![("_api_key", &api_key), ("page", page)];

        let url = PGYER_API_ENDPOINT.to_owned() + "listMy";
        let res = request(pairs, &url).await.unwrap();
        let build_code = res.get("code").unwrap().as_i64().unwrap() as i32;
        if build_code != 0 {
            println!("{}", res.get("message").unwrap());
            process::exit(0);
        }

        pretty_json(res.get("data").unwrap());

        process::exit(0);
    }

    fn pretty_json(str: &Value) {
        let formatted_json = serde_json::to_string_pretty(str);
        println!("{}", formatted_json.unwrap());
    }

    pub async fn get_build_info(build_key: &str) -> Value {
        let api_key = get_api_key();
        let pairs: Vec<(&str, &str)> = vec![("_api_key", &api_key), ("buildKey", build_key)];
        let url = format!(
            "{}buildInfo?_api_key={}&buildKey={}",
            PGYER_API_ENDPOINT, api_key, build_key
        );
        let res = request(pairs, &url).await.unwrap();
        return res;
    }

    pub async fn print_build_info(build_key: &str) {
        let build_info = get_build_info(build_key).await;
        pretty_json(build_info.get("data").unwrap());
    }

    fn create_form(form_fields: HashMap<&str, &str>) -> Form {
        let mut multipart_form = multipart::Form::new();
        for (key, value) in form_fields {
            multipart_form = multipart_form.text(key.to_string(), value.to_string());
        }
        multipart_form
    }

    async fn request(pairs: Vec<(&str, &str)>, url: &str) -> Result<Value> {
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
}
