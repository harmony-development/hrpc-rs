(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.5/http/request/struct.Request.html\" title=\"struct http::request::Request\">Request</a>&lt;Body&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/handler/struct.Handler.html\" title=\"struct hrpc::server::handler::Handler\">Handler</a>","synthetic":false,"types":["hrpc::server::handler::Handler"]},{"text":"impl <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.5/http/request/struct.Request.html\" title=\"struct http::request::Request\">Request</a>&lt;Body&gt;&gt; for <a class=\"struct\" href=\"hrpc/server/router/struct.RoutesFinalized.html\" title=\"struct hrpc::server::router::RoutesFinalized\">RoutesFinalized</a>","synthetic":false,"types":["hrpc::server::router::RoutesFinalized"]},{"text":"impl&lt;T, S:&nbsp;<a class=\"trait\" href=\"hrpc/server/trait.Service.html\" title=\"trait hrpc::server::Service\">Service</a>&gt; <a class=\"trait\" href=\"https://docs.rs/tower-service/0.3.1/tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;T&gt; for <a class=\"struct\" href=\"hrpc/server/struct.IntoMakeService.html\" title=\"struct hrpc::server::IntoMakeService\">IntoMakeService</a>&lt;S&gt;","synthetic":false,"types":["hrpc::server::IntoMakeService"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()