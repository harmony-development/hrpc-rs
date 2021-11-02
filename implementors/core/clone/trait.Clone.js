(function() {var implementors = {};
implementors["hello_world"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hello_world/hello/struct.WelcomeUserRequest.html\" title=\"struct hello_world::hello::WelcomeUserRequest\">WelcomeUserRequest</a>","synthetic":false,"types":["hello_world::hello::WelcomeUserRequest"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hello_world/hello/struct.WelcomeUserResponse.html\" title=\"struct hello_world::hello::WelcomeUserResponse\">WelcomeUserResponse</a>","synthetic":false,"types":["hello_world::hello::WelcomeUserResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hello_world/hello/greeter_client/struct.GreeterClient.html\" title=\"struct hello_world::hello::greeter_client::GreeterClient\">GreeterClient</a>","synthetic":false,"types":["hello_world::hello::greeter_client::GreeterClient"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"hello_world/hello/greeter_server/trait.Greeter.html\" title=\"trait hello_world::hello::greeter_server::Greeter\">Greeter</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hello_world/hello/greeter_server/struct.GreeterServer.html\" title=\"struct hello_world::hello::greeter_server::GreeterServer\">GreeterServer</a>&lt;T&gt;","synthetic":false,"types":["hello_world::hello::greeter_server::GreeterServer"]}];
implementors["hrpc"] = [{"text":"impl&lt;Req, Resp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/client/socket/struct.Socket.html\" title=\"struct hrpc::client::socket::Socket\">Socket</a>&lt;Req, Resp&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Req: <a class=\"trait\" href=\"https://docs.rs/prost/0.9.0/prost/message/trait.Message.html\" title=\"trait prost::message::Message\">Message</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Resp: <a class=\"trait\" href=\"https://docs.rs/prost/0.9.0/prost/message/trait.Message.html\" title=\"trait prost::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + 'static,&nbsp;</span>","synthetic":false,"types":["hrpc::client::socket::Socket"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/client/struct.Client.html\" title=\"struct hrpc::client::Client\">Client</a>","synthetic":false,"types":["hrpc::client::Client"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/server/handler/struct.HrpcLayer.html\" title=\"struct hrpc::server::handler::HrpcLayer\">HrpcLayer</a>","synthetic":false,"types":["hrpc::server::handler::HrpcLayer"]},{"text":"impl&lt;Req, Resp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/server/socket/struct.Socket.html\" title=\"struct hrpc::server::socket::Socket\">Socket</a>&lt;Req, Resp&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Req: <a class=\"trait\" href=\"https://docs.rs/prost/0.9.0/prost/message/trait.Message.html\" title=\"trait prost::message::Message\">Message</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;Resp: <a class=\"trait\" href=\"https://docs.rs/prost/0.9.0/prost/message/trait.Message.html\" title=\"trait prost::message::Message\">Message</a> + 'static,&nbsp;</span>","synthetic":false,"types":["hrpc::server::socket::Socket"]},{"text":"impl&lt;L, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/server/struct.LayeredService.html\" title=\"struct hrpc::server::LayeredService\">LayeredService</a>&lt;L, S&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;L: <a class=\"trait\" href=\"https://docs.rs/tower-layer/0.3.1/tower_layer/trait.Layer.html\" title=\"trait tower_layer::Layer\">Layer</a>&lt;<a class=\"struct\" href=\"hrpc/server/handler/struct.Handler.html\" title=\"struct hrpc::server::handler::Handler\">Handler</a>, Service = <a class=\"struct\" href=\"hrpc/server/handler/struct.Handler.html\" title=\"struct hrpc::server::handler::Handler\">Handler</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"hrpc/server/trait.Service.html\" title=\"trait hrpc::server::Service\">Service</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::LayeredService"]},{"text":"impl&lt;Outer, Inner&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/server/struct.ServiceStack.html\" title=\"struct hrpc::server::ServiceStack\">ServiceStack</a>&lt;Outer, Inner&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Outer: <a class=\"trait\" href=\"hrpc/server/trait.Service.html\" title=\"trait hrpc::server::Service\">Service</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Inner: <a class=\"trait\" href=\"hrpc/server/trait.Service.html\" title=\"trait hrpc::server::Service\">Service</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,&nbsp;</span>","synthetic":false,"types":["hrpc::server::ServiceStack"]},{"text":"impl&lt;S:&nbsp;<a class=\"trait\" href=\"hrpc/server/trait.Service.html\" title=\"trait hrpc::server::Service\">Service</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/server/struct.IntoMakeService.html\" title=\"struct hrpc::server::IntoMakeService\">IntoMakeService</a>&lt;S&gt;","synthetic":false,"types":["hrpc::server::IntoMakeService"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc/proto/struct.Error.html\" title=\"struct hrpc::proto::Error\">Error</a>","synthetic":false,"types":["hrpc::proto::Error"]}];
implementors["hrpc_build"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"hrpc_build/struct.Builder.html\" title=\"struct hrpc_build::Builder\">Builder</a>","synthetic":false,"types":["hrpc_build::prost::Builder"]}];
implementors["interop"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"interop/struct.Ping.html\" title=\"struct interop::Ping\">Ping</a>","synthetic":false,"types":["interop::Ping"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"interop/struct.Pong.html\" title=\"struct interop::Pong\">Pong</a>","synthetic":false,"types":["interop::Pong"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"interop/mu_client/struct.MuClient.html\" title=\"struct interop::mu_client::MuClient\">MuClient</a>","synthetic":false,"types":["interop::mu_client::MuClient"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"interop/mu_server/trait.Mu.html\" title=\"trait interop::mu_server::Mu\">Mu</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"interop/mu_server/struct.MuServer.html\" title=\"struct interop::mu_server::MuServer\">MuServer</a>&lt;T&gt;","synthetic":false,"types":["interop::mu_server::MuServer"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"interop/struct.MuService.html\" title=\"struct interop::MuService\">MuService</a>","synthetic":false,"types":["interop::MuService"]}];
implementors["server"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.56.1/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"server/struct.GreeterService.html\" title=\"struct server::GreeterService\">GreeterService</a>","synthetic":false,"types":["server::GreeterService"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()