(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl Debug for Client","synthetic":false,"types":[]},{"text":"impl&lt;Msg:&nbsp;Debug + Message + Default&gt; Debug for SocketMessage&lt;Msg&gt;","synthetic":false,"types":[]},{"text":"impl&lt;Msg:&nbsp;Debug, Resp:&nbsp;Debug&gt; Debug for Socket&lt;Msg, Resp&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Msg: Message,<br>&nbsp;&nbsp;&nbsp;&nbsp;Resp: Message + Default,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Debug for ClientError","synthetic":false,"types":[]},{"text":"impl Debug for InvalidUrlKind","synthetic":false,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl Debug for Builder","synthetic":false,"types":[]}];
implementors["interop"] = [{"text":"impl Debug for Ping","synthetic":false,"types":[]},{"text":"impl Debug for Pong","synthetic":false,"types":[]},{"text":"impl Debug for MuClient","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()