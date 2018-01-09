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

#[derive(Deserialize, Serialize)]
pub struct Image {
    pub id: i32,
    pub raw_id: Option<String>,
    pub sidecar_id: Option<String>,
    pub image_id: Option<String>,
    pub user_id: i32,
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
                    let resp_data =
                        serde_json::from_slice::<Image>(body.to_vec().as_slice()).unwrap();
                    info!("image {} added to library", resp_data.id);
                    Ok(resp_data.id)
                })
                .map_err(|e| {
                    warn!("adding image to library failed: {}", e);
                    e
                }),
        )
    }

    pub fn get_image(&self, image_id: i32) -> Box<Future<Item = Image, Error = error::Error>> {
        info!("loading image data from library");
        let uri = self.build_uri(&format!("/images/{}", image_id));

        Box::new(
            self.client
                .get(uri)
                .and_then(|response| match response.status() {
                    StatusCode::Ok => Ok(response),
                    _ => Err(error::Error::Status),
                })
                .and_then(|response| response.body().concat2())
                .and_then(|body| {
                    let resp_data =
                        serde_json::from_slice::<Image>(body.to_vec().as_slice()).unwrap();
                    info!("image {} loaded from library", resp_data.id);
                    Ok(resp_data)
                })
                .map_err(|e| {
                    warn!("loading image from library failed: {}", e);
                    e
                }),
        )
    }

    pub fn update_image(&self, image: Image) -> Box<Future<Item = i32, Error = error::Error>> {
        info!("updateing image {} in the library", image.id);
        let uri = self.build_uri(&format!("/images/{}", image.id));
        let mut req = Request::new(Method::Put, uri);
        req.set_body(serde_json::to_string(&image).unwrap());

        Box::new(
            self.client
                .request(req)
                .and_then(|response| match response.status() {
                    StatusCode::Ok => Ok(response),
                    _ => Err(error::Error::Status),
                })
                .and_then(|response| response.body().concat2())
                .and_then(|body| {
                    let resp_data =
                        serde_json::from_slice::<Image>(body.to_vec().as_slice()).unwrap();
                    info!("image {} updated in library", resp_data.id);
                    Ok(resp_data.id)
                })
                .map_err(move |e| {
                    warn!("updating image {} in library failed: {}", image.id, e);
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
    #[ignore]
    fn add_image() {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = LibraryClient::new("http://localhost:8002".parse().unwrap(), &handle);

        let new_image = NewImageData {
            raw_id: None,
            sidecar_id: None,
            image_id: None,
            user_id: 1,
        };
        let work = client.add_image(new_image).map_err(|e| panic!("{:?}", e));

        core.run(work).unwrap();
    }

    #[test]
    #[ignore]
    fn get_image() {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let client = LibraryClient::new("http://localhost:8002".parse().unwrap(), &handle);

        let work = client.get_image(123).map_err(|e| panic!("{:?}", e));

        core.run(work).unwrap();
    }

}
