(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/client/layer/modify/struct.ModifyLayer.html\" title=\"struct hrpc::client::layer::modify::ModifyLayer\">ModifyLayer</a>","synthetic":false,"types":["hrpc::client::layer::modify::ModifyLayer"]},{"text":"impl&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/layer/trace/struct.TraceLayer.html\" title=\"struct hrpc::server::layer::trace::TraceLayer\">TraceLayer</a>&lt;SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;SpanFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>) -&gt; <a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnRequestFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>, &amp;<a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">HrpcError</a>) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::layer::trace::TraceLayer"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/service/struct.HrpcLayer.html\" title=\"struct hrpc::server::service::HrpcLayer\">HrpcLayer</a> <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"type\" href=\"hrpc/request/type.BoxRequest.html\" title=\"type hrpc::request::BoxRequest\">BoxRequest</a>, Response = <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, Error = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;S::<a class=\"type\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::service::HrpcLayer"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;<a class=\"struct\" href=\"hrpc/server/service/struct.HrpcService.html\" title=\"struct hrpc::server::service::HrpcService\">HrpcService</a>&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttpLayer.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttpLayer\">HrpcServiceToHttpLayer</a>","synthetic":false,"types":["hrpc::server::transport::http::impl::HrpcServiceToHttpLayer"]},{"text":"impl&lt;S&gt; <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;S&gt; for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatusLayer.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer\">ErrorIdentifierToStatusLayer</a>","synthetic":false,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()