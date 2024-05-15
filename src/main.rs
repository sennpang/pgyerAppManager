use pgyer_uploader::app;
#[tokio::main]
async fn main() {
    app::check_params();
    let matches = app::get_command_params();
    if matches.value_of("file").is_some() {
        app::upload().await;
    }

    if matches.is_present("check") {
        app::check_proxy().await;
    }

    if matches.is_present("list") {
        app::get_app_list(matches.value_of("page").unwrap_or("1")).await;
    }

    if matches.value_of("appKey").is_some() {
        app::delete_app(&matches).await;
    }

    if matches.value_of("buildKey").is_some() {
        app::delete_build(&matches).await;
    }

    if matches.value_of("info").is_some() {
        app::print_build_info(&matches.value_of("info").unwrap()).await;
    }
}
