//! Layer to ratelimit a hRPC service and return errors respecting the hRPC
//! protocol.
//!
//! This layer allows you to control what is used as a key for storing state.
//! For example, this can allow you to use seperate state for seperate
//! connections based on [`std::net::SocketAddr`] or similar.
//!
//! You can extract and bypass keys using `ExtractKey` and `BypassForKey`
//! functions which you can specify on [`RateLimitLayer`] and [`RateLimit`].
//! An example can be found on [`RateLimitLayer::set_key_fns`] documentation.
//!
//! Note that if you don't specify anything and use the default configuration,
//! this essentially acts as a [`tower::limit::RateLimit`].

use pin_project_lite::pin_project;
use std::{
    collections::HashMap,
    convert::Infallible,
    future::Future,
    hash::Hash,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};

use crate::{
    encode,
    proto::{Error as HrpcError, RetryInfo},
    request::BoxRequest,
};

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
///
/// Read module documentation for more information.
#[derive(Clone)]
pub struct RateLimitLayer<ExtractKey, BypassForKey> {
    rate: Rate,
    extract_key: ExtractKey,
    bypass_for_key: BypassForKey,
}

type ExtractKeyDefault = fn(&mut BoxRequest) -> Option<()>;
type BypassForKeyDefault = fn(&()) -> bool;

impl RateLimitLayer<ExtractKeyDefault, BypassForKeyDefault> {
    /// Create new rate limit layer.
    pub fn new(num: u64, per: Duration) -> Self {
        let rate = Rate::new(num, per);
        RateLimitLayer {
            rate,
            extract_key: |_| None,
            bypass_for_key: |_| false,
        }
    }
}

impl<ExtractKey, BypassForKey> RateLimitLayer<ExtractKey, BypassForKey> {
    /// Set the key extraction and bypass functions.
    ///
    /// ```
    /// # use hrpc::server::layer::ratelimit::RateLimitLayer;
    /// # use std::{time::Duration, net::SocketAddr};
    ///
    /// // create a rate limit layer that uses SocketAddr as keys
    /// // to distinguish connections and use seperate state for them
    /// let layer = RateLimitLayer::new(5, Duration::from_secs(10))
    ///     .set_key_fns(
    ///         // extract ip addr from request
    ///         |req| req.extensions().get::<SocketAddr>().map(|addr| addr.ip()),
    ///         // bypass ratelimit for loopback ips
    ///         |key| key.is_loopback(),
    ///     );
    /// ```
    pub fn set_key_fns<NewBypassForKey, NewExtractKeyFn, NewKey>(
        self,
        extract: NewExtractKeyFn,
        bypass: NewBypassForKey,
    ) -> RateLimitLayer<NewExtractKeyFn, NewBypassForKey>
    where
        NewBypassForKey: Fn(&NewKey) -> bool + Clone,
        NewExtractKeyFn: Fn(&mut BoxRequest) -> Option<NewKey> + Clone,
        NewKey: Eq + Hash,
    {
        RateLimitLayer {
            rate: self.rate,
            bypass_for_key: bypass,
            extract_key: extract,
        }
    }
}

impl<BypassForKey, ExtractKey, Key, S> Layer<S> for RateLimitLayer<ExtractKey, BypassForKey>
where
    ExtractKey: Fn(&mut BoxRequest) -> Option<Key> + Clone,
    BypassForKey: Fn(&Key) -> bool + Clone,
    Key: Eq + Hash,
{
    type Service = RateLimit<S, ExtractKey, BypassForKey, Key>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimit::new(
            service,
            self.rate,
            self.extract_key.clone(),
            self.bypass_for_key.clone(),
        )
    }
}

/// Enforces a rate limit on the number of requests the underlying
/// service can handle over a period of time.
///
/// Read module documentation for more information.
pub struct RateLimit<T, ExtractKey, BypassForKey, Key> {
    inner: T,
    rate: Rate,
    global_state: State,
    keyed_states: HashMap<Key, State>,
    extract_key: ExtractKey,
    bypass_for_key: BypassForKey,
}

#[derive(Debug)]
enum State {
    // The service has hit its limit
    Limited { after: Instant },
    Ready { until: Instant, rem: u64 },
}

impl State {
    fn new_ready(rate: &Rate) -> Self {
        State::Ready {
            rem: rate.num(),
            until: Instant::now(),
        }
    }
}

impl<S, ExtractKey, BypassForKey, Key> RateLimit<S, ExtractKey, BypassForKey, Key>
where
    ExtractKey: Fn(&mut BoxRequest) -> Option<Key>,
    BypassForKey: Fn(&Key) -> bool,
    Key: Eq + Hash,
{
    /// Create a new rate limiter.
    pub fn new(
        inner: S,
        rate: Rate,
        extract_key: ExtractKey,
        bypass_for_key: BypassForKey,
    ) -> Self {
        RateLimit {
            inner,
            global_state: State::new_ready(&rate),
            rate,
            extract_key,
            bypass_for_key,
            keyed_states: HashMap::new(),
        }
    }

    /// Get a reference to the inner service.
    pub fn get_ref(&self) -> &S {
        &self.inner
    }

    /// Get a mutable reference to the inner service.
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Consume `self`, returning the inner service.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S, ExtractKey, BypassForKey, Key> Service<BoxRequest>
    for RateLimit<S, ExtractKey, BypassForKey, Key>
where
    S: Service<BoxRequest, Response = BoxResponse, Error = Infallible>,
    ExtractKey: Fn(&mut BoxRequest) -> Option<Key>,
    BypassForKey: Fn(&Key) -> bool,
    Key: Eq + Hash,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = RateLimitFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut request: BoxRequest) -> Self::Future {
        let state = match (self.extract_key)(&mut request) {
            Some(key) => {
                if (self.bypass_for_key)(&key) {
                    let fut = Service::call(&mut self.inner, request);
                    return RateLimitFuture::ready(fut);
                }

                self.keyed_states
                    .entry(key)
                    .or_insert_with(|| State::new_ready(&self.rate))
            }
            None => &mut self.global_state,
        };

        match *state {
            State::Ready { mut until, mut rem } => {
                let now = Instant::now();

                // If the period has elapsed, reset it.
                if now >= until {
                    until = now + self.rate.per();
                    rem = self.rate.num();
                }

                if rem > 1 {
                    rem -= 1;
                    *state = State::Ready { until, rem };
                } else {
                    // The service is disabled until further notice
                    let after = Instant::now() + self.rate.per();
                    *state = State::Limited { after };
                }

                // Call the inner future
                let fut = Service::call(&mut self.inner, request);
                RateLimitFuture::ready(fut)
            }
            State::Limited { after } => {
                let now = Instant::now();
                if now < after {
                    tracing::trace!("rate limit exceeded.");
                    let after = after - now;
                    return RateLimitFuture::limited(after);
                }

                // Reset state
                *state = State::Ready {
                    until: now + self.rate.per(),
                    rem: self.rate.num(),
                };

                // Call the inner future
                let fut = Service::call(&mut self.inner, request);
                RateLimitFuture::ready(fut)
            }
        }
    }
}

pin_project! {
    #[project = EnumProj]
    enum RateLimitFutureInner<Fut> {
        Ready { #[pin] fut: Fut },
        Limited { after: Duration },
    }
}

pin_project! {
    /// Future for [`RateLimit`].
    pub struct RateLimitFuture<Fut> {
        #[pin]
        inner: RateLimitFutureInner<Fut>,
    }
}

impl<Fut> RateLimitFuture<Fut> {
    fn ready(fut: Fut) -> Self {
        Self {
            inner: RateLimitFutureInner::Ready { fut },
        }
    }

    fn limited(after: Duration) -> Self {
        Self {
            inner: RateLimitFutureInner::Limited { after },
        }
    }
}

impl<Fut> Future for RateLimitFuture<Fut>
where
    Fut: Future<Output = Result<BoxResponse, Infallible>>,
{
    type Output = Result<BoxResponse, Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            EnumProj::Ready { fut } => fut.poll(cx),
            EnumProj::Limited { after } => {
                let retry_after = after.as_secs_f64().ceil() as u32;
                let retry_info = RetryInfo { retry_after };

                let err = HrpcError::new_resource_exhausted("rate limited; please try later")
                    .with_details(encode::encode_protobuf_message(&retry_info).freeze());

                Poll::Ready(Ok(err.into()))
            }
        }
    }
}

use crate::response::BoxResponse;

#[doc(inline)]
pub use self::rate::Rate;

mod rate {
    use std::time::Duration;

    /// A rate of requests per time period.
    #[derive(Debug, Copy, Clone)]
    pub struct Rate {
        num: u64,
        per: Duration,
    }

    impl Rate {
        /// Create a new rate.
        ///
        /// # Panics
        ///
        /// This function panics if `num` or `per` is 0.
        pub fn new(num: u64, per: Duration) -> Self {
            assert!(num > 0);
            assert!(per > Duration::from_millis(0));

            Rate { num, per }
        }

        pub(crate) fn num(&self) -> u64 {
            self.num
        }

        pub(crate) fn per(&self) -> Duration {
            self.per
        }
    }
}
