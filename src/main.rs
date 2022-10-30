use cyberpunk::{
    endpoint::{Endpoint, Transformation},
    telemetry::{get_subscriber, init_subscriber},
};

fn main() {
    let subscriber = get_subscriber("cyberpunk".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let mut endpoint = Endpoint::new();
    endpoint.audio = "celtic_pt2.mp3";

    let transaction = Transformation::Concat("reversed_celtic_pt2.mp3".to_string());
    endpoint.pipeline.push(transaction);

    endpoint.process().unwrap();
}
