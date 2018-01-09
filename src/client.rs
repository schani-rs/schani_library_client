use futures::{Future, Stream};
use hyper::{Client, Method, StatusCode, Uri};
use hyper::client::{HttpConnector, Request};
use hyper::error;
use serde_json;
use tokio_core::reactor::Handle;

pub struct LibraryClient {
    uri: Uri,
    client: Client<HttpConnector>,
}

#[derive(Serialize)]
pub struct NewImageData {
    pub raw_id: Option<String>,
    pub sidecar_id: Option<String>,
    pub image_id: Option<String>,
    pub user_id: i32,
}

#[derive(Deserialize)]
pub struct NewImageResponseData {
    id: i32,
    //raw_id: String,
    //sidecar_id: String,
    //image_id: String,
    //user_id: i32,
}

impl LibraryClient {
    pub fn new(uri: Uri, handle: &Handle) -> Self {
        LibraryClient {
            uri: uri,
            client: Client::new(handle),
        }
    }

    fn build_uri(&self, path: &str) -> Uri {
        let mut uri_str = self.uri.to_string();
        uri_str.push_str(path);
        uri_str.parse().unwrap()
    }

    pub fn add_image(&self, data: NewImageData) -> Box<Future<Item = i32, Error = error::Error>> {
        info!("adding a new image to the library");
        let uri = self.build_uri("/images");
        let mut req = Request::new(Method::Post, uri);
        req.set_body(serde_json::to_string(&data).unwrap());

        Box::new(
            self.client
                .request(req)
                .and_then(|response| match response.status() {
                    StatusCode::Ok => Ok(response),
                    _ => Err(error::Error::Status),
                })
                .and_then(|response| response.body().concat2())
                .and_then(|body| {
                    let resp_data = serde_json::from_slice::<NewImageResponseData>(
                        body.to_vec().as_slice(),
                    ).unwrap();
                    info!("image {} added to library", resp_data.id);
                    Ok(resp_data.id)
                })
                .map_err(|e| {
                    warn!("adding image to library failed: {}", e);
                    e
                }),
        )
    }
}

#[cfg(test)]
mod tests {
    use tokio_core::reactor::Core;

    use super::*;

    #[test]
    fn add_image() {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = StoreClient::new("http://localhost:8000".parse().unwrap(), &handle);

        let work = client
            .upload_raw_image(b"123".to_vec())
            .map_err(|e| panic!("{:?}", e));

        core.run(work).unwrap();
    }

}
