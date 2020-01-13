use super::boxed::{Data, Payload};
use futures::{future, Future, Poll};
use linkerd2_error::Error;
use linkerd2_stack::Proxy;

#[derive(Copy, Clone, Debug)]
pub struct Layer;

#[derive(Clone, Debug)]
pub struct BoxResponse<S>(S);

impl<S> tower::layer::Layer<S> for Layer {
    type Service = BoxResponse<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BoxResponse(inner)
    }
}

impl<Req, B, S, P> Proxy<Req, S> for BoxResponse<P>
where
    P: Proxy<Req, S, Response = http::Response<B>>,
    S: tower::Service<P::Request>,
    B: hyper::body::Payload<Data = Data, Error = Error> + Send + 'static,
{
    type Request = P::Request;
    type Response = http::Response<Payload>;
    type Error = P::Error;
    type Future = future::Map<P::Future, fn(P::Response) -> Self::Response>;

    fn proxy(&self, inner: &mut S, req: Req) -> Self::Future {
        self.0.proxy(inner, req).map(|rsp| rsp.map(Payload::new))
    }
}

impl<S, Req, B> tower::Service<Req> for BoxResponse<S>
where
    S: tower::Service<Req, Response = http::Response<B>>,
    B: hyper::body::Payload<Data = Data, Error = Error> + Send + 'static,
{
    type Response = http::Response<Payload>;
    type Error = S::Error;
    type Future = future::Map<S::Future, fn(S::Response) -> Self::Response>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.0.poll_ready()
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.0.call(req).map(|rsp| rsp.map(Payload::new))
    }
}
