(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/client/layer/backoff/struct.BackoffLayer.html\" title=\"struct hrpc::client::layer::backoff::BackoffLayer\">BackoffLayer</a>","synthetic":false,"types":["hrpc::client::layer::backoff::BackoffLayer"]},{"text":"impl&lt;BypassForKey, ExtractKey, Key, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/layer/ratelimit/struct.RateLimitLayer.html\" title=\"struct hrpc::server::layer::ratelimit::RateLimitLayer\">RateLimitLayer</a>&lt;ExtractKey, BypassForKey&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;ExtractKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;mut <a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;Key&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;BypassForKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.reference.html\">&amp;</a>Key) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.bool.html\">bool</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Key: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::layer::ratelimit::RateLimitLayer"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/service/struct.HrpcLayer.html\" title=\"struct hrpc::server::service::HrpcLayer\">HrpcLayer</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;S::<a class=\"associatedtype\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::service::HrpcLayer"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;<a class=\"struct\" href=\"hrpc/server/service/struct.HrpcService.html\" title=\"struct hrpc::server::service::HrpcService\">HrpcService</a>&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttpLayer.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttpLayer\">HrpcServiceToHttpLayer</a>","synthetic":false,"types":["hrpc::server::transport::http::impl::HrpcServiceToHttpLayer"]},{"text":"impl&lt;ToStatus, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatusLayer.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer\">ErrorIdentifierToStatusLayer</a>&lt;ToStatus&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;ToStatus: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer"]},{"text":"impl&lt;ModifyReq, ModifyResp, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/common/layer/modify/struct.ModifyLayer.html\" title=\"struct hrpc::common::layer::modify::ModifyLayer\">ModifyLayer</a>&lt;ModifyReq, ModifyResp&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;ModifyReq: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;ModifyResp: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::modify::ModifyLayer"]},{"text":"impl&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/common/layer/trace/struct.TraceLayer.html\" title=\"struct hrpc::common::layer::trace::TraceLayer\">TraceLayer</a>&lt;SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;SpanFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>) -&gt; <a class=\"struct\" href=\"https://docs.rs/tracing/0.1.32/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnRequestFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.32/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.32/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.32/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>, &amp;<a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">HrpcError</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::trace::TraceLayer"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()