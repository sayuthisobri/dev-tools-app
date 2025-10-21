use crate::errors::aws_error::AwsResult;
use crate::services::aws::AwsClient;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::operation::list_buckets::ListBucketsOutput;
use aws_sdk_s3::operation::list_objects::ListObjectsOutput;
use aws_sdk_s3::types::{Bucket, Object, Owner};
pub use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct S3Bucket {
    /// <p>The name of the bucket.</p>
    pub name: ::std::option::Option<::std::string::String>,
    /// <p>Date the bucket was created. This date can change when making changes to your bucket, such as editing its bucket policy.</p>
    pub creation_date: ::std::option::Option<String>,
    /// <p><code>BucketRegion</code> indicates the Amazon Web Services region where the bucket is located. If the request contains at least one valid parameter, it is included in the response.</p>
    pub bucket_region: ::std::option::Option<::std::string::String>,
    /// <p>The Amazon Resource Name (ARN) of the S3 bucket. ARNs uniquely identify Amazon Web Services resources across all of Amazon Web Services.</p><note>
    /// <p>This parameter is only supported for S3 directory buckets. For more information, see <a href="https://docs.aws.amazon.com/AmazonS3/latest/userguide/directory-buckets-tagging.html">Using tags with directory buckets</a>.</p>
    /// </note>
    pub bucket_arn: ::std::option::Option<::std::string::String>,
}

impl From<&Bucket> for S3Bucket {
    fn from(b: &Bucket) -> Self {
        S3Bucket {
            name: b.name.clone(),
            creation_date: b.creation_date().map(|d| d.to_string()),
            bucket_region: b.bucket_region().map(|r| r.to_string()),
            bucket_arn: b.bucket_arn().map(|arn| arn.to_string()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct S3Object {
    pub key: Option<String>,
    pub size: Option<i64>,
    pub last_modified: Option<String>,
    pub e_tag: Option<String>,
    pub storage_class: Option<String>,
    pub owner: Option<S3Owner>,
}

impl From<&Object> for S3Object {
    fn from(o: &Object) -> Self {
        S3Object {
            key: o.key().map(|k| k.to_string()),
            size: o.size(),
            last_modified: o.last_modified().map(|d| d.to_string()),
            e_tag: o.e_tag().map(|e| e.to_string()),
            storage_class: o.storage_class().map(|s| s.to_string()),
            owner: o.owner().map(|o| o.into()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct S3ObjectMetadata {
    pub key: Option<String>,
    pub content_length: Option<i64>,
    pub size_formatted: String,
    pub content_type: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub e_tag: Option<String>,
    pub storage_class: Option<String>,
    pub expires: Option<String>,
    pub content_disposition: Option<String>,
    pub meta: Option<HashMap<String, String>>,
}

impl S3ObjectMetadata {}

fn format_file_size(size: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as i64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

impl From<GetObjectOutput> for S3ObjectMetadata {
    fn from(o: GetObjectOutput) -> Self {
        S3ObjectMetadata {
            key: None,
            content_length: o.content_length(),
            size_formatted: o
                .content_length()
                .map(|s| format_file_size(s))
                .unwrap_or("-".to_string()),
            content_type: o.content_type().map(|ct| ct.to_string()),
            last_modified: o.last_modified().map(|dt| {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_default()
            }),
            e_tag: o.e_tag().map(|e| e.to_string()),
            storage_class: o.storage_class().map(|s| s.to_string()),
            expires: o.expires_string().map(|e| e.to_string()),
            content_disposition: o.content_disposition().map(|cd| cd.to_string()),
            meta: o.metadata().map(|m| m.clone()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PageableList<T> {
    pub items: Vec<T>,
    pub next_token: Option<String>,
    pub owner: Option<S3Owner>,
    pub prefix: Option<String>,
}

impl From<ListObjectsOutput> for PageableList<S3Object> {
    fn from(output: ListObjectsOutput) -> Self {
        PageableList {
            items: output.contents().iter().map(|i| i.into()).collect(),
            next_token: output.next_marker().map(|m| m.into()),
            owner: None,
            prefix: output.prefix().map(|p| p.into()),
        }
    }
}

impl From<ListBucketsOutput> for PageableList<S3Bucket> {
    fn from(output: ListBucketsOutput) -> Self {
        PageableList {
            items: output.buckets().iter().map(|b| b.into()).collect(),
            next_token: output.continuation_token().map(|o| o.into()),
            owner: output.owner().map(|o| o.into()),
            prefix: output.prefix().map(|p| p.into()),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct S3Owner {
    pub id: Option<String>,
    pub display_name: Option<String>,
}

impl From<&Owner> for S3Owner {
    fn from(owner: &Owner) -> Self {
        S3Owner {
            id: owner.id.clone(),
            display_name: owner.display_name.clone(),
        }
    }
}

pub async fn get_s3_client(profile: &str) -> S3Client {
    let config = aws_config::from_env().profile_name(profile).load().await;
    S3Client::new(&config)
}

impl AwsClient {
    pub async fn prepare_s3(&mut self) -> () {
        if self.s3.is_none() {
            self.s3 = Some(get_s3_client(self.profile.as_str()).await);
        }
    }

    pub fn get_s3_client(&self) -> &S3Client {
        self.s3.as_ref().expect("s3 client not initialized")
    }

    pub async fn list_buckets(&self) -> AwsResult<PageableList<S3Bucket>> {
        Ok(self
            .get_s3_client()
            .list_buckets()
            .send()
            .await
            .map(|o| o.into())?)
    }

    pub async fn list_objects(&self, bucket: &str) -> AwsResult<PageableList<S3Object>> {
        Ok(self
            .get_s3_client()
            .list_objects()
            .bucket(bucket)
            .send()
            .await
            .map(|o| o.into())?)
    }

    pub async fn download_object(&self, bucket: &str, key: &str) -> AwsResult<S3ObjectMetadata> {
        Ok(self
            .get_s3_client()
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|o| S3ObjectMetadata::from(o))?)
    }
}
