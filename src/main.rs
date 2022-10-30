use cyberpunk::{
    endpoint::{Endpoint, Transformation},
    telemetry::{get_subscriber, init_subscriber}, configuration::get_configuration, startup::Application,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("cyberpunk".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration.clone()).await?;
    application.run_until_stopped().await?;

    let mut endpoint = Endpoint::new();
    endpoint.audio = "celtic_pt2.mp3";

    let transaction = Transformation::Concat("reversed_celtic_pt2.mp3".to_string());
    endpoint.pipeline.push(transaction);

    endpoint.process().unwrap();

    Ok(())
}
