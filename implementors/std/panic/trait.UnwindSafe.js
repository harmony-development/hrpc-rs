(function() {var implementors = {};
implementors["hrpc"] = [{"text":"impl !UnwindSafe for ClientError","synthetic":true,"types":[]},{"text":"impl UnwindSafe for InvalidUrlKind","synthetic":true,"types":[]},{"text":"impl !UnwindSafe for Client","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; !UnwindSafe for Socket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; !UnwindSafe for ReadSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;Msg, Resp&gt; !UnwindSafe for WriteSocket&lt;Msg, Resp&gt;","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for Request&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: UnwindSafe,&nbsp;</span>","synthetic":true,"types":[]}];
implementors["hrpc_build"] = [{"text":"impl UnwindSafe for Builder","synthetic":true,"types":[]}];
implementors["interop"] = [{"text":"impl !UnwindSafe for MuClient","synthetic":true,"types":[]},{"text":"impl&lt;T&gt; UnwindSafe for MuServer&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: RefUnwindSafe,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Ping","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Pong","synthetic":true,"types":[]},{"text":"impl UnwindSafe for Server","synthetic":true,"types":[]},{"text":"impl UnwindSafe for ServerError","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()