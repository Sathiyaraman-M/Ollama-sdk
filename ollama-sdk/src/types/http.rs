use crate::Result;
use bytes::Bytes;
use serde::Serialize;

/// Represents a generic HTTP request.
///
/// This struct is used internally by the transport layer to construct
/// and send requests.
#[derive(Default, Debug)]
pub struct HttpRequest {
    /// The URL path of the API endpoint (e.g., "/api/chat").
    pub url: String,
    /// The HTTP verb (GET, POST, PUT, DELETE) for the request.
    pub verb: HttpVerb,
    /// The optional request body, serialized as a JSON value.
    pub body: Option<serde_json::Value>,
}

/// Represents the HTTP verbs supported for requests.
#[derive(Default, Debug)]
pub enum HttpVerb {
    /// HTTP GET method.
    #[default]
    GET,
    /// HTTP POST method.
    POST,
    /// HTTP PUT method.
    PUT,
    /// HTTP DELETE method.
    DELETE,
}

/// Represents a generic HTTP response.
///
/// This struct contains the raw bytes of the response body.
#[derive(Debug)]
pub struct HttpResponse {
    /// The optional body of the HTTP response.
    pub body: Option<Bytes>,
}

impl HttpRequest {
    /// Creates a new [`HttpRequest`] with the specified URL path.
    ///
    /// The default HTTP verb is [`HttpVerb::GET`].
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Sets the HTTP verb for the request to [`HttpVerb::GET`].
    pub fn get(mut self) -> Self {
        self.verb = HttpVerb::GET;
        self
    }

    /// Sets the HTTP verb for the request to [`HttpVerb::POST`].
    pub fn post(mut self) -> Self {
        self.verb = HttpVerb::POST;
        self
    }

    /// Sets the HTTP verb for the request to [`HttpVerb::PUT`].
    pub fn put(mut self) -> Self {
        self.verb = HttpVerb::PUT;
        self
    }

    /// Sets the HTTP verb for the request to [`HttpVerb::DELETE`].
    pub fn delete(mut self) -> Self {
        self.verb = HttpVerb::DELETE;
        self
    }

    /// Sets the request body by serializing the given `T` into a JSON value.
    ///
    /// # Arguments
    ///
    /// * `body` - The data structure to be serialized as the request body.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::JsonParse`](variant@crate::Error::JsonParse) if the
    /// `body` cannot be serialized to JSON.
    pub fn body<T: Serialize>(mut self, body: T) -> Result<Self> {
        self.body = Some(serde_json::to_value(body)?);
        Ok(self)
    }
}
