// Copyright 2021 Datafuse Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::future::Future;
use std::panic::Location;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use futures::FutureExt;

use crate::error::Result;
use crate::error_code::ErrorCode;

pub struct JoinHandle<Output> {
    inner: tokio::task::JoinHandle<Output>,
}

impl<Output> JoinHandle<Output> {
    pub fn create(inner: tokio::task::JoinHandle<Output>) -> Self {
        Self { inner }
    }

    pub fn abort(&self) {
        self.inner.abort();
    }
}

impl<Output> Future for JoinHandle<Output> {
    type Output = Result<Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => match res {
                Ok(res) => Poll::Ready(Ok(res)),
                Err(error) => match error.is_panic() {
                    true => {
                        let cause = error.into_panic();
                        Poll::Ready(Err(match cause.downcast_ref::<&'static str>() {
                            None => match cause.downcast_ref::<String>() {
                                None => ErrorCode::PanicError("Sorry, unknown panic message"),
                                Some(message) => ErrorCode::PanicError(message.to_string()),
                            },
                            Some(message) => ErrorCode::PanicError(message.to_string()),
                        }))
                    }
                    false => Poll::Ready(Err(ErrorCode::TokioError("Tokio task is cancelled"))),
                },
            },
        }
    }
}

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[expect(clippy::disallowed_methods)]
    tokio::spawn(location_future(future, Location::caller(), None))
}

fn location_future<F>(
    future: F,
    frame_location: &'static Location,
    frame_name: Option<String>,
) -> impl Future<Output = F::Output>
where
    F: Future,
{
    let frame_name = if let Some(n) = frame_name {
        n
    } else {
        std::any::type_name::<F>()
            .trim_end_matches("::{{closure}}")
            .to_string()
    };

    async_backtrace::location!().frame(future)
}
