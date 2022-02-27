(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/client/boxed/struct.BoxedTransport.html\" title=\"struct hrpc::client::boxed::BoxedTransport\">BoxedTransport</a>","synthetic":false,"types":["hrpc::client::boxed::BoxedTransport"]},{"text":"impl&lt;S, Err&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/client/layer/backoff/struct.Backoff.html\" title=\"struct hrpc::client::layer::backoff::Backoff\">Backoff</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Error = <a class=\"enum\" href=\"hrpc/client/transport/enum.TransportError.html\" title=\"enum hrpc::client::transport::TransportError\">TransportError</a>&lt;Err&gt;&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::client::layer::backoff::Backoff"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/client/transport/http/hyper/struct.Hyper.html\" title=\"struct hrpc::client::transport::http::hyper::Hyper\">Hyper</a>","synthetic":false,"types":["hrpc::client::transport::http::hyper::Hyper"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/client/transport/http/wasm/struct.Wasm.html\" title=\"struct hrpc::client::transport::http::wasm::Wasm\">Wasm</a>","synthetic":false,"types":["hrpc::client::transport::http::wasm::Wasm"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/client/transport/mock/struct.Mock.html\" title=\"struct hrpc::client::transport::mock::Mock\">Mock</a>","synthetic":false,"types":["hrpc::client::transport::mock::Mock"]},{"text":"impl&lt;S, ExtractKey, BypassForKey, Key&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/layer/ratelimit/struct.RateLimit.html\" title=\"struct hrpc::server::layer::ratelimit::RateLimit\">RateLimit</a>&lt;S, ExtractKey, BypassForKey, Key&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ExtractKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;mut <a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;Key&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;BypassForKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.reference.html\">&amp;</a>Key) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.bool.html\">bool</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Key: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/hash/trait.Hash.html\" title=\"trait core::hash::Hash\">Hash</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::layer::ratelimit::RateLimit"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/router/struct.RoutesFinalized.html\" title=\"struct hrpc::server::router::RoutesFinalized\">RoutesFinalized</a>","synthetic":false,"types":["hrpc::server::router::RoutesFinalized"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/service/struct.HrpcService.html\" title=\"struct hrpc::server::service::HrpcService\">HrpcService</a>","synthetic":false,"types":["hrpc::server::service::HrpcService"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.6/http/request/struct.Request.html\" title=\"struct http::request::Request\">Request</a>&lt;Body&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttp.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttp\">HrpcServiceToHttp</a>","synthetic":false,"types":["hrpc::server::transport::http::impl::HrpcServiceToHttp"]},{"text":"impl&lt;M:&nbsp;<a class=\"trait\" href=\"hrpc/server/trait.MakeRoutes.html\" title=\"trait hrpc::server::MakeRoutes\">MakeRoutes</a>, L, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;&amp;'_ AddrStream&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.MakeRoutesToHttp.html\" title=\"struct hrpc::server::transport::http::impl::MakeRoutesToHttp\">MakeRoutesToHttp</a>&lt;M, L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;<a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttp.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttp\">HrpcServiceToHttp</a>, Service = S&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/server/transport/http/type.HttpRequest.html\" title=\"type hrpc::server::transport::http::HttpRequest\">HttpRequest</a>, Response = <a class=\"type\" href=\"hrpc/server/transport/http/type.HttpResponse.html\" title=\"type hrpc::server::transport::http::HttpResponse\">HttpResponse</a>, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;S::<a class=\"associatedtype\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::transport::http::impl::MakeRoutesToHttp"]},{"text":"impl&lt;ToStatus, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatus.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatus\">ErrorIdentifierToStatus</a>&lt;ToStatus, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ToStatus: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.str.html\">str</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.59.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.6/http/status/struct.StatusCode.html\" title=\"struct http::status::StatusCode\">StatusCode</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatus"]},{"text":"impl&lt;T, S:&nbsp;<a class=\"trait\" href=\"hrpc/server/trait.MakeRoutes.html\" title=\"trait hrpc::server::MakeRoutes\">MakeRoutes</a>&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;T&gt; for <a class=\"struct\" href=\"hrpc/server/struct.IntoMakeService.html\" title=\"struct hrpc::server::IntoMakeService\">IntoMakeService</a>&lt;S&gt;","synthetic":false,"types":["hrpc::server::IntoMakeService"]},{"text":"impl&lt;ModifyReq, ModifyResp, S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/common/layer/modify/struct.Modify.html\" title=\"struct hrpc::common::layer::modify::Modify\">Modify</a>&lt;ModifyReq, ModifyResp, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ModifyReq: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;mut <a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>),<br>&nbsp;&nbsp;&nbsp;&nbsp;ModifyResp: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;mut <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::modify::Modify"]},{"text":"impl&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.59.0/std/primitive.unit.html\">()</a>&gt;&gt; for <a class=\"struct\" href=\"hrpc/common/layer/trace/struct.Trace.html\" title=\"struct hrpc::common::layer::trace::Trace\">Trace</a>&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;SpanFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>) -&gt; <a class=\"struct\" href=\"https://docs.rs/tracing/0.1.31/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnRequestFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.31/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>),<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.31/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.31/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>, &amp;<a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">HrpcError</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.59.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::trace::Trace"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()