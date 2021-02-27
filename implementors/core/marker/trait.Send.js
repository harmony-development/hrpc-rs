(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl Send for ClientError","synthetic":true,"types":[]},{"text":"impl Send for InvalidUrlKind","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Send for Request&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Send,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Send for Client","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Send for Socket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Send for ReadSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Send for WriteSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl Send for Builder","synthetic":true,"types":[]}];
implementors["interop"] = [{"text":"impl Send for MuClient","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Send for MuServer&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Send for Ping","synthetic":true,"types":[]},{"text":"impl Send for Pong","synthetic":true,"types":[]},{"text":"impl Send for Server","synthetic":true,"types":[]},{"text":"impl Send for ServerError","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()