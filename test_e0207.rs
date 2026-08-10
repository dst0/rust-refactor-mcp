pub trait Body {}
pub struct Request<T>(T);
pub struct Response<T>(T);

pub type MyResponse<T> = Response<T>;

pub trait Service<Request> {
    type Response;
}

mod sealed {
    pub trait Sealed<T> {}
}

pub mod my_mod {
    // Deliberately cause E0603
    pub trait HttpService<ReqBody>: super::sealed::Sealed<ReqBody> {
        type ResBody: super::Body;
    }
}

use my_mod::HttpService;

impl<T, B1, B2> HttpService<B1> for T
where
    T: Service<Request<B1>, Response = MyResponse<B2>>,
    B2: Body,
{
    type ResBody = B2;
}

impl<T, B1, B2> sealed::Sealed<B1> for T
where
    T: Service<Request<B1>, Response = MyResponse<B2>>,
    B2: Body,
{}

fn main() {}
