(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl Unpin for ClientError","synthetic":true,"types":[]},{"text":"impl Unpin for InvalidUrlKind","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; Unpin for Socket&lt;Msg, Resp&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Msg: Unpin,<br>&nbsp;&nbsp;&nbsp;&nbsp;Resp: Unpin,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl Unpin for Builder","synthetic":true,"types":[]}];
implementors["interop"] = [{"text":"impl Unpin for MuClient","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; Unpin for MuServer&lt;T&gt;","synthetic":true,"types":[]},{"text":"impl Unpin for Ping","synthetic":true,"types":[]},{"text":"impl Unpin for Pong","synthetic":true,"types":[]},{"text":"impl Unpin for Server","synthetic":true,"types":[]},{"text":"impl Unpin for ServerError","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()