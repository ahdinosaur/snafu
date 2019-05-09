#![feature(async_await)]

mod api {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub enum Error {
        InvalidUrl { url: String }
    }

    pub async fn fetch_page(url: &str) -> Result<String, Error> {
        InvalidUrl { url }.fail()
    }
}

use futures::future;
use snafu::{Snafu, ResultExt, futures_std::FutureExt};

#[derive(Debug, Snafu)]
enum Error {
    UnableToLoadAppleStock { source: api::Error },
    UnableToLoadGoogleStock { source: api::Error },
}

// Normal `Result` code works with `await`
async fn load_stock_data_sequential() -> Result<String, Error> {
    let apple = api::fetch_page("apple").await.context(UnableToLoadAppleStock)?;
    let google = api::fetch_page("google").await.context(UnableToLoadGoogleStock)?;
    Ok(format!("{}+{}", apple, google))
}

// move to gen
impl snafu::IntoError for UnableToLoadAppleStock {
    type SourceError = api::Error;
    type Error = Error;

    fn into_error(self, error: Self::SourceError) -> Self::Error {
        snafu::Context { context: self, error }.into()
    }
}

impl snafu::IntoError for UnableToLoadGoogleStock {
    type SourceError = api::Error;
    type Error = Error;

    fn into_error(self, error: Self::SourceError) -> Self::Error {
        snafu::Context { context: self, error }.into()
    }
}

// Can also use a combinator
async fn load_stock_data_concurrent() -> Result<String, Error> {
    let apple = api::fetch_page("apple").context(UnableToLoadAppleStock);
    let google = api::fetch_page("google").context(UnableToLoadGoogleStock);

    let (apple, google) = future::try_join(apple, google).await?;

    Ok(format!("{}+{}", apple, google))
}


fn main() {
    fn check<T: std::error::Error>() {}
    check::<Error>();

    let a = futures::executor::block_on(load_stock_data_sequential());
    a.unwrap_err();

    let b = futures::executor::block_on(load_stock_data_concurrent());
    b.unwrap_err();
}
