(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl Send for ClientError","synthetic":true,"types":[]},{"text":"impl Send for InvalidUrlKind","synthetic":true,"types":[]},{"text":"impl Send for Client","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Send for Socket&lt;Msg, Resp&gt;","synthetic":true,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl Send for Builder","synthetic":true,"types":[]}];
implementors["interop"] = [{"text":"impl Send for MuClient","synthetic":true,"types":[]},{"text":"impl Send for Ping","synthetic":true,"types":[]},{"text":"impl Send for Pong","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()