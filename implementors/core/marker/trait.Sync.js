(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl Sync for ClientError","synthetic":true,"types":[]},{"text":"impl Sync for InvalidUrlKind","synthetic":true,"types":[]},{"text":"impl Sync for Client","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Sync for Socket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Sync for ReadSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Sync for WriteSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Sync for Request&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Sync,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl Sync for Builder","synthetic":true,"types":[]}];
implementors["interop"] = [{"text":"impl Sync for MuClient","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Sync for MuServer&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Sync for Ping","synthetic":true,"types":[]},{"text":"impl Sync for Pong","synthetic":true,"types":[]},{"text":"impl Sync for Server","synthetic":true,"types":[]},{"text":"impl Sync for ServerError","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()