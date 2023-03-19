use aws_sdk_s3::{types::ByteStream, Client, Config};
use aws_smithy_http::body::{BoxBody, SdkBody};
use axum::http::HeaderMap;
use axum::BoxError;
use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryStreamExt};
use http_body::Body;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{self, Poll},
};
use sync_wrapper::SyncWrapper;

pin_project! {
    struct StreamBody<S> {
        #[pin]
        inner: SyncWrapper<S>,
    }
}

impl<S> Body for StreamBody<S>
where
    S: Stream<Item = super::Result<Bytes>>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        this.inner.get_pin_mut().poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// S3-backed storage
// is cloning this expensive??
#[derive(Clone)]
pub struct Storage {
    bucket_name: String,
    client: Client,
}

impl Storage {
    /// Create a new storage instance
    #[must_use]
    pub fn new(bucket_name: String, config: Config) -> Self {
        Self {
            bucket_name,
            client: Client::from_conf(config),
        }
    }
}

impl Storage {
    pub async fn delete(&self, path: &str) -> super::Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get(&self, path: &str) -> super::Result<BoxStream<'static, super::Result<Bytes>>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await?;

        Ok(response.body.map_err(Into::into).boxed())
    }

    pub async fn put(
        &self,
        path: &str,
        input_stream: impl Stream<Item = super::Result<Bytes>> + 'static + Send,
        content_length: i64,
        content_type: &str,
    ) -> super::Result<()> {
        let body = BoxBody::new(StreamBody {
            inner: SyncWrapper::new(input_stream),
        });

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(path)
            .content_length(content_length)
            .content_type(content_type)
            .body(ByteStream::new(SdkBody::from_dyn(body)))
            .send()
            .await?;

        Ok(())
    }
}
