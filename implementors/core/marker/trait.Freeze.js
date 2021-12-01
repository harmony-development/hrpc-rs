(function() {var implementors = {};
implementors["chat"] = [{"text":"impl&lt;Inner&gt; Freeze for <a class=\"struct\" href=\"chat/chat/chat_client/struct.ChatClient.html\" title=\"struct chat::chat::chat_client::ChatClient\">ChatClient</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,&nbsp;</span>","synthetic":true,"types":["chat::chat::chat_client::ChatClient"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"chat/chat/chat_server/struct.ChatServer.html\" title=\"struct chat::chat::chat_server::ChatServer\">ChatServer</a>&lt;T&gt;","synthetic":true,"types":["chat::chat::chat_server::ChatServer"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chat/chat/struct.Message.html\" title=\"struct chat::chat::Message\">Message</a>","synthetic":true,"types":["chat::chat::Message"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chat/chat/struct.Empty.html\" title=\"struct chat::chat::Empty\">Empty</a>","synthetic":true,"types":["chat::chat::Empty"]}];
implementors["chat_common"] = [{"text":"impl&lt;Inner&gt; Freeze for <a class=\"struct\" href=\"chat_common/chat/chat_client/struct.ChatClient.html\" title=\"struct chat_common::chat::chat_client::ChatClient\">ChatClient</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,&nbsp;</span>","synthetic":true,"types":["chat_common::chat::chat_client::ChatClient"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"chat_common/chat/chat_server/struct.ChatServer.html\" title=\"struct chat_common::chat::chat_server::ChatServer\">ChatServer</a>&lt;T&gt;","synthetic":true,"types":["chat_common::chat::chat_server::ChatServer"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chat_common/chat/struct.Message.html\" title=\"struct chat_common::chat::Message\">Message</a>","synthetic":true,"types":["chat_common::chat::Message"]},{"text":"impl Freeze for <a class=\"struct\" href=\"chat_common/chat/struct.Empty.html\" title=\"struct chat_common::chat::Empty\">Empty</a>","synthetic":true,"types":["chat_common::chat::Empty"]}];
implementors["chat_server"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"chat_server/struct.ChatService.html\" title=\"struct chat_server::ChatService\">ChatService</a>","synthetic":true,"types":["chat_server::ChatService"]}];
implementors["chat_wasm_client"] = [{"text":"impl !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"enum\" href=\"chat_wasm_client/enum.Msg.html\" title=\"enum chat_wasm_client::Msg\">Msg</a>","synthetic":true,"types":["chat_wasm_client::Msg"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"chat_wasm_client/struct.Model.html\" title=\"struct chat_wasm_client::Model\">Model</a>","synthetic":true,"types":["chat_wasm_client::Model"]},{"text":"impl !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"chat_wasm_client/struct.Props.html\" title=\"struct chat_wasm_client::Props\">Props</a>","synthetic":true,"types":["chat_wasm_client::Props"]}];
implementors["hello_world"] = [{"text":"impl&lt;Inner&gt; Freeze for <a class=\"struct\" href=\"hello_world/hello/greeter_client/struct.GreeterClient.html\" title=\"struct hello_world::hello::greeter_client::GreeterClient\">GreeterClient</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,&nbsp;</span>","synthetic":true,"types":["hello_world::hello::greeter_client::GreeterClient"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"hello_world/hello/greeter_server/struct.GreeterServer.html\" title=\"struct hello_world::hello::greeter_server::GreeterServer\">GreeterServer</a>&lt;T&gt;","synthetic":true,"types":["hello_world::hello::greeter_server::GreeterServer"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hello_world/hello/struct.WelcomeUserRequest.html\" title=\"struct hello_world::hello::WelcomeUserRequest\">WelcomeUserRequest</a>","synthetic":true,"types":["hello_world::hello::WelcomeUserRequest"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hello_world/hello/struct.WelcomeUserResponse.html\" title=\"struct hello_world::hello::WelcomeUserResponse\">WelcomeUserResponse</a>","synthetic":true,"types":["hello_world::hello::WelcomeUserResponse"]}];
implementors["hrpc"] = [{"text":"impl&lt;TransportError&gt; !Freeze for <a class=\"enum\" href=\"hrpc/client/error/enum.ClientError.html\" title=\"enum hrpc::client::error::ClientError\">ClientError</a>&lt;TransportError&gt;","synthetic":true,"types":["hrpc::client::error::ClientError"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/client/layer/modify/struct.ModifyLayer.html\" title=\"struct hrpc::client::layer::modify::ModifyLayer\">ModifyLayer</a>","synthetic":true,"types":["hrpc::client::layer::modify::ModifyLayer"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"hrpc/client/layer/modify/struct.Modify.html\" title=\"struct hrpc::client::layer::modify::Modify\">Modify</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::client::layer::modify::Modify"]},{"text":"impl&lt;Fut&gt; Freeze for <a class=\"struct\" href=\"hrpc/client/layer/modify/struct.ModifyFuture.html\" title=\"struct hrpc::client::layer::modify::ModifyFuture\">ModifyFuture</a>&lt;Fut&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::client::layer::modify::ModifyFuture"]},{"text":"impl !Freeze for <a class=\"struct\" href=\"hrpc/client/transport/http/hyper/struct.Hyper.html\" title=\"struct hrpc::client::transport::http::hyper::Hyper\">Hyper</a>","synthetic":true,"types":["hrpc::client::transport::http::hyper::Hyper"]},{"text":"impl !Freeze for <a class=\"struct\" href=\"hrpc/client/transport/http/hyper/struct.HyperCallFuture.html\" title=\"struct hrpc::client::transport::http::hyper::HyperCallFuture\">HyperCallFuture</a>","synthetic":true,"types":["hrpc::client::transport::http::hyper::HyperCallFuture"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/http/hyper/enum.HyperError.html\" title=\"enum hrpc::client::transport::http::hyper::HyperError\">HyperError</a>","synthetic":true,"types":["hrpc::client::transport::http::hyper::HyperError"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/http/hyper/enum.SocketInitError.html\" title=\"enum hrpc::client::transport::http::hyper::SocketInitError\">SocketInitError</a>","synthetic":true,"types":["hrpc::client::transport::http::hyper::SocketInitError"]},{"text":"impl !Freeze for <a class=\"struct\" href=\"hrpc/client/transport/http/wasm/struct.Wasm.html\" title=\"struct hrpc::client::transport::http::wasm::Wasm\">Wasm</a>","synthetic":true,"types":["hrpc::client::transport::http::wasm::Wasm"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/client/transport/http/wasm/struct.SocketProtocols.html\" title=\"struct hrpc::client::transport::http::wasm::SocketProtocols\">SocketProtocols</a>","synthetic":true,"types":["hrpc::client::transport::http::wasm::SocketProtocols"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/http/wasm/enum.WasmError.html\" title=\"enum hrpc::client::transport::http::wasm::WasmError\">WasmError</a>","synthetic":true,"types":["hrpc::client::transport::http::wasm::WasmError"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/client/transport/http/wasm/struct.CallFuture.html\" title=\"struct hrpc::client::transport::http::wasm::CallFuture\">CallFuture</a>","synthetic":true,"types":["hrpc::client::transport::http::wasm::CallFuture"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/http/enum.InvalidServerUrl.html\" title=\"enum hrpc::client::transport::http::InvalidServerUrl\">InvalidServerUrl</a>","synthetic":true,"types":["hrpc::client::transport::http::InvalidServerUrl"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/client/transport/mock/struct.Mock.html\" title=\"struct hrpc::client::transport::mock::Mock\">Mock</a>","synthetic":true,"types":["hrpc::client::transport::mock::Mock"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/client/transport/mock/struct.MockCallFuture.html\" title=\"struct hrpc::client::transport::mock::MockCallFuture\">MockCallFuture</a>","synthetic":true,"types":["hrpc::client::transport::mock::MockCallFuture"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/mock/enum.MockError.html\" title=\"enum hrpc::client::transport::mock::MockError\">MockError</a>","synthetic":true,"types":["hrpc::client::transport::mock::MockError"]},{"text":"impl&lt;Err&gt; !Freeze for <a class=\"enum\" href=\"hrpc/client/transport/enum.TransportError.html\" title=\"enum hrpc::client::transport::TransportError\">TransportError</a>&lt;Err&gt;","synthetic":true,"types":["hrpc::client::transport::TransportError"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/enum.TransportRequest.html\" title=\"enum hrpc::client::transport::TransportRequest\">TransportRequest</a>","synthetic":true,"types":["hrpc::client::transport::TransportRequest"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/client/transport/enum.TransportResponse.html\" title=\"enum hrpc::client::transport::TransportResponse\">TransportResponse</a>","synthetic":true,"types":["hrpc::client::transport::TransportResponse"]},{"text":"impl&lt;Inner&gt; Freeze for <a class=\"struct\" href=\"hrpc/client/struct.Client.html\" title=\"struct hrpc::client::Client\">Client</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::client::Client"]},{"text":"impl&lt;SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/layer/trace/struct.TraceLayer.html\" title=\"struct hrpc::server::layer::trace::TraceLayer\">TraceLayer</a>&lt;SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnRequestFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;SpanFn: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::layer::trace::TraceLayer"]},{"text":"impl&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/layer/trace/struct.Trace.html\" title=\"struct hrpc::server::layer::trace::Trace\">Trace</a>&lt;S, SpanFn, OnRequestFn, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnRequestFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;SpanFn: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::layer::trace::Trace"]},{"text":"impl&lt;Fut, OnSuccessFn, OnErrorFn&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/layer/trace/struct.TraceFuture.html\" title=\"struct hrpc::server::layer::trace::TraceFuture\">TraceFuture</a>&lt;Fut, OnSuccessFn, OnErrorFn&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnErrorFn: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;OnSuccessFn: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::layer::trace::TraceFuture"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/router/struct.Routes.html\" title=\"struct hrpc::server::router::Routes\">Routes</a>","synthetic":true,"types":["hrpc::server::router::Routes"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/router/struct.RoutesFinalized.html\" title=\"struct hrpc::server::router::RoutesFinalized\">RoutesFinalized</a>","synthetic":true,"types":["hrpc::server::router::RoutesFinalized"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/service/struct.HrpcService.html\" title=\"struct hrpc::server::service::HrpcService\">HrpcService</a>","synthetic":true,"types":["hrpc::server::service::HrpcService"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/service/struct.HrpcLayer.html\" title=\"struct hrpc::server::service::HrpcLayer\">HrpcLayer</a>","synthetic":true,"types":["hrpc::server::service::HrpcLayer"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttp.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttp\">HrpcServiceToHttp</a>","synthetic":true,"types":["hrpc::server::transport::http::impl::HrpcServiceToHttp"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.HrpcServiceToHttpLayer.html\" title=\"struct hrpc::server::transport::http::impl::HrpcServiceToHttpLayer\">HrpcServiceToHttpLayer</a>","synthetic":true,"types":["hrpc::server::transport::http::impl::HrpcServiceToHttpLayer"]},{"text":"impl&lt;S, L&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/impl/struct.MakeRoutesToHttp.html\" title=\"struct hrpc::server::transport::http::impl::MakeRoutesToHttp\">MakeRoutesToHttp</a>&lt;S, L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::transport::http::impl::MakeRoutesToHttp"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatusLayer.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer\">ErrorIdentifierToStatusLayer</a>","synthetic":true,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusLayer"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatus.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatus\">ErrorIdentifierToStatus</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatus"]},{"text":"impl&lt;Fut&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/layer/errid_to_status/struct.ErrorIdentifierToStatusFuture.html\" title=\"struct hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusFuture\">ErrorIdentifierToStatusFuture</a>&lt;Fut&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Fut: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::transport::http::layer::errid_to_status::ErrorIdentifierToStatusFuture"]},{"text":"impl&lt;L&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/transport/http/struct.Hyper.html\" title=\"struct hrpc::server::transport::http::Hyper\">Hyper</a>&lt;L&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::transport::http::Hyper"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/server/transport/mock/struct.Mock.html\" title=\"struct hrpc::server::transport::mock::Mock\">Mock</a>","synthetic":true,"types":["hrpc::server::transport::mock::Mock"]},{"text":"impl&lt;S, L, M&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/struct.LayeredService.html\" title=\"struct hrpc::server::LayeredService\">LayeredService</a>&lt;S, L, M&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;M: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::LayeredService"]},{"text":"impl&lt;Outer, Inner&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/struct.ServiceStack.html\" title=\"struct hrpc::server::ServiceStack\">ServiceStack</a>&lt;Outer, Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;Outer: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::ServiceStack"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"hrpc/server/struct.IntoMakeService.html\" title=\"struct hrpc::server::IntoMakeService\">IntoMakeService</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::server::IntoMakeService"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/body/struct.Body.html\" title=\"struct hrpc::body::Body\">Body</a>","synthetic":true,"types":["hrpc::body::Body"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/common/extensions/struct.Extensions.html\" title=\"struct hrpc::common::extensions::Extensions\">Extensions</a>","synthetic":true,"types":["hrpc::common::extensions::Extensions"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"hrpc/common/future/struct.Ready.html\" title=\"struct hrpc::common::future::Ready\">Ready</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::common::future::Ready"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/common/socket/enum.SocketMessage.html\" title=\"enum hrpc::common::socket::SocketMessage\">SocketMessage</a>","synthetic":true,"types":["hrpc::common::socket::SocketMessage"]},{"text":"impl !Freeze for <a class=\"enum\" href=\"hrpc/common/socket/enum.SocketError.html\" title=\"enum hrpc::common::socket::SocketError\">SocketError</a>","synthetic":true,"types":["hrpc::common::socket::SocketError"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/common/socket/struct.CombineError.html\" title=\"struct hrpc::common::socket::CombineError\">CombineError</a>","synthetic":true,"types":["hrpc::common::socket::CombineError"]},{"text":"impl&lt;Req, Resp&gt; Freeze for <a class=\"struct\" href=\"hrpc/common/socket/struct.Socket.html\" title=\"struct hrpc::common::socket::Socket\">Socket</a>&lt;Req, Resp&gt;","synthetic":true,"types":["hrpc::common::socket::Socket"]},{"text":"impl&lt;Resp&gt; Freeze for <a class=\"struct\" href=\"hrpc/common/socket/struct.ReadSocket.html\" title=\"struct hrpc::common::socket::ReadSocket\">ReadSocket</a>&lt;Resp&gt;","synthetic":true,"types":["hrpc::common::socket::ReadSocket"]},{"text":"impl&lt;Req&gt; Freeze for <a class=\"struct\" href=\"hrpc/common/socket/struct.WriteSocket.html\" title=\"struct hrpc::common::socket::WriteSocket\">WriteSocket</a>&lt;Req&gt;","synthetic":true,"types":["hrpc::common::socket::WriteSocket"]},{"text":"impl&lt;S&gt; Freeze for <a class=\"struct\" href=\"hrpc/common/transport/tokio_tungstenite/struct.WebSocket.html\" title=\"struct hrpc::common::transport::tokio_tungstenite::WebSocket\">WebSocket</a>&lt;S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>","synthetic":true,"types":["hrpc::common::transport::tokio_tungstenite::WebSocket"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/common/transport/ws_wasm/struct.WebSocket.html\" title=\"struct hrpc::common::transport::ws_wasm::WebSocket\">WebSocket</a>","synthetic":true,"types":["hrpc::common::transport::ws_wasm::WebSocket"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/common/transport/mock/struct.MockSender.html\" title=\"struct hrpc::common::transport::mock::MockSender\">MockSender</a>","synthetic":true,"types":["hrpc::common::transport::mock::MockSender"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/common/transport/mock/struct.MockReceiver.html\" title=\"struct hrpc::common::transport::mock::MockReceiver\">MockReceiver</a>","synthetic":true,"types":["hrpc::common::transport::mock::MockReceiver"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/decode/enum.DecodeBodyError.html\" title=\"enum hrpc::decode::DecodeBodyError\">DecodeBodyError</a>","synthetic":true,"types":["hrpc::decode::DecodeBodyError"]},{"text":"impl !Freeze for <a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">Error</a>","synthetic":true,"types":["hrpc::proto::Error"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/proto/struct.RetryInfo.html\" title=\"struct hrpc::proto::RetryInfo\">RetryInfo</a>","synthetic":true,"types":["hrpc::proto::RetryInfo"]},{"text":"impl Freeze for <a class=\"enum\" href=\"hrpc/proto/enum.HrpcErrorIdentifier.html\" title=\"enum hrpc::proto::HrpcErrorIdentifier\">HrpcErrorIdentifier</a>","synthetic":true,"types":["hrpc::proto::HrpcErrorIdentifier"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/proto/struct.NotHrpcErrorIdentifier.html\" title=\"struct hrpc::proto::NotHrpcErrorIdentifier\">NotHrpcErrorIdentifier</a>","synthetic":true,"types":["hrpc::proto::NotHrpcErrorIdentifier"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/request/struct.Parts.html\" title=\"struct hrpc::request::Parts\">Parts</a>","synthetic":true,"types":["hrpc::request::Parts"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"hrpc/struct.Request.html\" title=\"struct hrpc::Request\">Request</a>&lt;T&gt;","synthetic":true,"types":["hrpc::request::Request"]},{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc/response/struct.Parts.html\" title=\"struct hrpc::response::Parts\">Parts</a>","synthetic":true,"types":["hrpc::response::Parts"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"hrpc/struct.Response.html\" title=\"struct hrpc::Response\">Response</a>&lt;T&gt;","synthetic":true,"types":["hrpc::response::Response"]}];
implementors["hrpc_build"] = [{"text":"impl Freeze for <a class=\"struct\" href=\"hrpc_build/struct.Builder.html\" title=\"struct hrpc_build::Builder\">Builder</a>","synthetic":true,"types":["hrpc_build::prost::Builder"]}];
implementors["interop"] = [{"text":"impl&lt;Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"interop/mu_client/struct.MuClient.html\" title=\"struct interop::mu_client::MuClient\">MuClient</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a>,&nbsp;</span>","synthetic":true,"types":["interop::mu_client::MuClient"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"interop/mu_server/struct.MuServer.html\" title=\"struct interop::mu_server::MuServer\">MuServer</a>&lt;T&gt;","synthetic":true,"types":["interop::mu_server::MuServer"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"interop/struct.Ping.html\" title=\"struct interop::Ping\">Ping</a>","synthetic":true,"types":["interop::Ping"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"interop/struct.Pong.html\" title=\"struct interop::Pong\">Pong</a>","synthetic":true,"types":["interop::Pong"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"interop/struct.MuService.html\" title=\"struct interop::MuService\">MuService</a>","synthetic":true,"types":["interop::MuService"]}];
implementors["mock"] = [{"text":"impl&lt;Inner&gt; Freeze for <a class=\"struct\" href=\"mock/hello/greeter_client/struct.GreeterClient.html\" title=\"struct mock::hello::greeter_client::GreeterClient\">GreeterClient</a>&lt;Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: Freeze,&nbsp;</span>","synthetic":true,"types":["mock::hello::greeter_client::GreeterClient"]},{"text":"impl&lt;T&gt; Freeze for <a class=\"struct\" href=\"mock/hello/greeter_server/struct.GreeterServer.html\" title=\"struct mock::hello::greeter_server::GreeterServer\">GreeterServer</a>&lt;T&gt;","synthetic":true,"types":["mock::hello::greeter_server::GreeterServer"]},{"text":"impl Freeze for <a class=\"struct\" href=\"mock/hello/struct.WelcomeUserRequest.html\" title=\"struct mock::hello::WelcomeUserRequest\">WelcomeUserRequest</a>","synthetic":true,"types":["mock::hello::WelcomeUserRequest"]},{"text":"impl Freeze for <a class=\"struct\" href=\"mock/hello/struct.WelcomeUserResponse.html\" title=\"struct mock::hello::WelcomeUserResponse\">WelcomeUserResponse</a>","synthetic":true,"types":["mock::hello::WelcomeUserResponse"]},{"text":"impl Freeze for <a class=\"struct\" href=\"mock/server/struct.GreeterService.html\" title=\"struct mock::server::GreeterService\">GreeterService</a>","synthetic":true,"types":["mock::server::GreeterService"]}];
implementors["server"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Freeze.html\" title=\"trait core::marker::Freeze\">Freeze</a> for <a class=\"struct\" href=\"server/struct.GreeterService.html\" title=\"struct server::GreeterService\">GreeterService</a>","synthetic":true,"types":["server::GreeterService"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()