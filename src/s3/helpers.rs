use aws_credential_types::Credentials;
use aws_sdk_s3::{Config, Region};
use std::env;

use super::interface::Storage;

pub fn setup_storage() -> Result<Storage, aws_sdk_s3::Error> {
    let bucket_name = env::var("BUCKET_NAME").expect("BUCKET_NAME env variable");
    let access_key = env::var("ACCESS_KEY").expect("ACCESS_KEY env variable");
    let secret_access_key = env::var("SECRET_ACCESS_KEY").expect("SECRET_ACCESS_KEY env variable");
    let endpoint_url = env::var("ENDPOINT_URL").expect("ENDPOINT_URL env variable");
    let region = env::var("REGION").expect("REGION env variable");

    let credentials = Credentials::from_keys(access_key, secret_access_key, None);
    let config = Config::builder()
        .region(Region::new(region))
        .force_path_style(true)
        .credentials_provider(credentials)
        .endpoint_url(endpoint_url)
        .build();

    Ok(Storage::new(bucket_name, config))
}
