use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_sqs::Client;

pub async fn get_sqs_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    return Client::new(&config);
}
