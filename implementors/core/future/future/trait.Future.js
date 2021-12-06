(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/client/transport/http/hyper/struct.HyperCallFuture.html\" title=\"struct hrpc::client::transport::http::hyper::HyperCallFuture\">HyperCallFuture</a>","synthetic":false,"types":["hrpc::client::transport::http::hyper::HyperCallFuture"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/client/transport/http/wasm/struct.CallFuture.html\" title=\"struct hrpc::client::transport::http::wasm::CallFuture\">CallFuture</a>","synthetic":false,"types":["hrpc::client::transport::http::wasm::CallFuture"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/client/transport/mock/struct.MockCallFuture.html\" title=\"struct hrpc::client::transport::mock::MockCallFuture\">MockCallFuture</a>","synthetic":false,"types":["hrpc::client::transport::mock::MockCallFuture"]},{"text":"impl&lt;Fut&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/server/layer/ratelimit/struct.RateLimitFuture.html\" title=\"struct hrpc::server::layer::ratelimit::RateLimitFuture\">RateLimitFuture</a>&lt;Fut&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["hrpc::server::layer::ratelimit::RateLimitFuture"]},{"text":"impl&lt;ToStatus, Fut, Err&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatusFuture.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusFuture\">ErrorIdentifierToStatusFuture</a>&lt;ToStatus, Fut&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, Err&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ToStatus: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.57.0/std/primitive.str.html\">str</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.5/http/status/struct.StatusCode.html\" title=\"struct http::status::StatusCode\">StatusCode</a>&gt;,&nbsp;</span>","synthetic":false,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusFuture"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/common/future/struct.Ready.html\" title=\"struct hrpc::common::future::Ready\">Ready</a>&lt;T&gt;","synthetic":false,"types":["hrpc::common::future::Ready"]},{"text":"impl&lt;ModifyResp, Fut, Err&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/common/layer/modify/struct.ModifyFuture.html\" title=\"struct hrpc::common::layer::modify::ModifyFuture\">ModifyFuture</a>&lt;ModifyResp, Fut&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, Err&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ModifyResp: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;mut <a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>),&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::modify::ModifyFuture"]},{"text":"impl&lt;Fut, FutErr, OnSuccessFn, OnErrorFn&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"hrpc/common/layer/trace/struct.TraceFuture.html\" title=\"struct hrpc::common::layer::trace::TraceFuture\">TraceFuture</a>&lt;Fut, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&lt;Output = <a class=\"enum\" href=\"https://doc.rust-lang.org/1.57.0/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, FutErr&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>),<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(&amp;<a class=\"type\" href=\"hrpc/response/type.BoxResponse.html\" title=\"type hrpc::response::BoxResponse\">BoxResponse</a>, &amp;<a class=\"struct\" href=\"https://docs.rs/tracing/0.1.29/tracing/span/struct.Span.html\" title=\"struct tracing::span::Span\">Span</a>, &amp;<a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">HrpcError</a>),&nbsp;</span>","synthetic":false,"types":["hrpc::common::layer::trace::TraceFuture"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()