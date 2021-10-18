var searchIndex = JSON.parse('{\
"client":{"doc":"","t":[5],"n":["main"],"q":["client"],"d":[""],"i":[0],"f":[[[],[["result",4,["boxerror"]],["boxerror",6]]]],"p":[]},\
"hello_world":{"doc":"","t":[6,0,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,11,12,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,8,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,11],"n":["BoxError","hello","WelcomeUserRequest","WelcomeUserResponse","borrow","borrow","borrow_mut","borrow_mut","clear","clear","clone","clone","clone_into","clone_into","default","default","encode_raw","encode_raw","encoded_len","encoded_len","eq","eq","fmt","fmt","from","from","greeter_client","greeter_server","into","into","into_request","into_request","into_response","into_response","merge_field","merge_field","ne","ne","to_owned","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","user_name","vzip","vzip","welcome_message","GreeterClient","borrow","borrow_mut","clone","clone_into","connect","fmt","from","inner","into","into_response","new_http","new_inner","to_owned","try_from","try_into","type_id","vzip","welcome_user","Greeter","GreeterServer","borrow","borrow_mut","clone","clone_into","fmt","from","into","into_response","make_routes","new","to_owned","try_from","try_into","type_id","vzip","welcome_user","welcome_user_middleware"],"q":["hello_world","","hello_world::hello","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hello_world::hello::greeter_client","","","","","","","","","","","","","","","","","","","hello_world::hello::greeter_server","","","","","","","","","","","","","","","","","",""],"d":["A boxed error.","<code>hello</code> package protobuf definitions and service.","Request used in <code>WelcomeUser</code> endpoint.","Response used in <code>WelcomeUser</code> endpoint.","","","","","","","","","","","","","","","","","","","","","","","Generated client implementations.","Generated server implementations.","","","","","","","","","","","","","","","","","","","The user name of the user you want to welcome.","","","The welcome message.","The greeter service.","","","","","","","","","","","","","","","","","","","Generated trait containing hRPC methods that should be …","The greeter service.","","","","","","","","","","Create a new service server.","","","","","","","Filter to be run before all API operations but after API …"],"i":[0,0,0,0,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,0,0,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,2,1,1,2,2,0,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,0,0,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,5,5],"f":[null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[],["welcomeuserrequest",3]],[[],["welcomeuserresponse",3]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["usize",15]],[[],["usize",15]],[[["welcomeuserrequest",3]],["bool",15]],[[["welcomeuserresponse",3]],["bool",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],null,null,[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["response",3]],[[],["response",3]],[[["wiretype",4],["u32",15],["decodecontext",3]],[["decodeerror",3],["result",4,["decodeerror"]]]],[[["wiretype",4],["u32",15],["decodecontext",3]],[["decodeerror",3],["result",4,["decodeerror"]]]],[[["welcomeuserrequest",3]],["bool",15]],[[["welcomeuserresponse",3]],["bool",15]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,[[]],[[]],null,null,[[]],[[]],[[],["greeterclient",3]],[[]],[[],["clientresult",6]],[[["formatter",3]],["result",6]],[[]],[[],["client",3]],[[]],[[],["response",3]],[[["httpclient",6],["uri",3]],["clientresult",6]],[[["client",3]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],[[]],null,null,[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[]],[[]],[[],["response",3]],[[],["routes",3]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],[[["hrpcrequest",3,["welcomeuserrequest"]],["welcomeuserrequest",3]],[["box",3,["future"]],["pin",3,["box"]]]],[[["str",15]],["hrpclayer",6]]],"p":[[3,"WelcomeUserRequest"],[3,"WelcomeUserResponse"],[3,"GreeterClient"],[3,"GreeterServer"],[8,"Greeter"]]},\
"hrpc":{"doc":"Common code used in hRPC code generation.","t":[6,4,17,6,6,8,8,13,13,3,3,14,14,14,0,11,11,11,11,11,11,0,14,11,5,5,0,11,11,11,11,11,11,11,11,11,11,14,11,11,11,11,11,10,11,10,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,6,6,5,5,5,3,6,11,11,11,11,11,11,0,11,11,11,5,11,11,11,11,11,0,11,11,11,11,11,13,13,4,6,13,13,13,13,13,13,3,13,3,13,13,4,13,13,3,13,13,13,4,13,13,13,13,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,23,6,3,3,8,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,11,11,11,0,11,11,11,11,11,11,11,11,10,11,11,11,0,11,0,11,11,11,11,11,11,11,11,11,11,11,11,0,11,11,11,13,13,13,13,8,13,13,13,13,13,13,4,6,4,13,13,13,13,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,6,3,11,11,11,11,11,11,11,11,5,11,11,11,11,11,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,8,10,10,5],"n":["BoxError","DecodeBodyError","HRPC_HEADER","HttpRequest","HttpResponse","IntoRequest","IntoResponse","InvalidBody","InvalidProtoMessage","Request","Response","bail","bail_result","bail_result_as_response","body","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","client","combine_services","empty","encode_protobuf_message","encode_protobuf_message_to","exports","fmt","fmt","fmt","from","from","from","from","from","header_map","header_map_mut","include_proto","into","into","into","into_message","into_message","into_request","into_request","into_response","into_response","into_response","into_response","into_response","new","new","server","source","to_string","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","vzip","vzip","vzip","BoxBody","HyperBody","box_body","empty_box_body","full_box_body","Client","HttpClient","borrow","borrow_mut","clone","clone_into","connect_socket","connect_socket_req","error","execute_request","fmt","from","http_client","into","into_response","modify_request_headers_with","new","new_inner","socket","to_owned","try_from","try_into","type_id","vzip","AlreadyClosed","Capacity","ClientError","ClientResult","ConnectionClosed","ContentNotSupported","EndpointError","FailedRequestBuilder","Http","Http","HttpError","HttpFormat","HyperError","InvalidScheme","InvalidUrl","InvalidUrlKind","Io","Io","IoError","MessageDecode","Protocol","SendQueueFull","SocketError","SocketError","Tls","Url","Utf8","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","cause","description","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_raw_os_error","get_mut","get_ref","get_ref","into","into","into","into","into","into_cause","into_inner","into_non_blocking","into_response","into_response","into_response","into_response","into_response","is","is_body_write_aborted","is_canceled","is_closed","is_connect","is_incomplete_message","is_parse","is_parse_status","is_parse_too_large","is_timeout","is_user","kind","last_os_error","new","raw_os_error","source","source","source","source","to_string","to_string","to_string","to_string","to_string","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","vzip","endpoint","raw_error","status","Socket","borrow","borrow_mut","clone","clone_into","close","fmt","from","into","into_response","is_closed","receive_message","send_message","spawn_process_task","spawn_task","to_owned","try_from","try_into","type_id","vzip","async_trait","HrpcLayer","IntoMakeService","LayeredServer","Server","ServerStack","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","call","clone","clone","clone","clone_into","clone_into","clone_into","combine_with","error","from","from","from","handler","into","into","into","into_make_service","into_response","into_response","into_response","layer","make_routes","make_routes","make_routes","poll_ready","router","serve","socket","to_owned","to_owned","to_owned","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","utils","vzip","vzip","vzip","AlreadyClosed","Capacity","ConnectionClosed","Custom","CustomError","DecodeBodyError","Http","HttpFormat","Io","Protocol","SendQueueFull","ServerError","ServerResult","SocketError","SocketError","Tls","Url","Utf8","as_error_response","as_status_message","borrow","borrow","borrow_mut","borrow_mut","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","into","into","into_non_blocking","into_response","into_response","into_response","json_err_bytes","source","source","to_string","to_string","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","CallFuture","Handler","borrow","borrow_mut","call","from","into","into_response","layer","new","not_found","poll_ready","try_from","try_into","type_id","vzip","Routes","RoutesFinalized","any","borrow","borrow","borrow_mut","borrow_mut","build","call","combine_with","default","from","from","into","into","into_response","into_response","layer","new","poll_ready","route","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","Socket","borrow","borrow_mut","clone","clone_into","close","fmt","from","into","into_response","is_closed","receive_message","send_message","spawn_process_task","spawn_task","to_owned","try_from","try_into","type_id","vzip","HeaderMapExt","header_contains_str","header_eq","serve"],"q":["hrpc","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::body","","","","","hrpc::client","","","","","","","","","","","","","","","","","","","","","","","","hrpc::client::error","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::client::error::ClientError","","","hrpc::client::socket","","","","","","","","","","","","","","","","","","","","hrpc::exports","hrpc::server","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::server::error","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::server::handler","","","","","","","","","","","","","","","","hrpc::server::router","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::server::socket","","","","","","","","","","","","","","","","","","","","hrpc::server::utils","","",""],"d":["Alias for a type-erased error type.","Errors that can occur while decoding the body of a <code>Request</code>…","The hRPC protobuf mimetype.","HTTP request used by hRPC.","HTTP response used by hRPC.","Trait used for blanket impls on generated protobuf types.","Trait used for converting any type to a Response type.","An error occured while reading the body.","The body contained an invalid protobuf message.","A hRPC request.","hRPC response type.","Bails with an error.","Takes a <code>Result</code>, returns the error if it’s <code>Err</code>, …","Takes a <code>Result</code>, returns the error as a HTTP response if it…","Body utitilies and types.","","","","","","","Common client types and functions.","Combines a list of services that implement <code>Server</code>.","Create an empty request.","Encodes a protobuf message into a new <code>BytesMut</code> buffer.","Encodes a protobuf message into the given <code>BytesMut</code> buffer.","Some re-exported crates that might be useful while …","","","","","","","","","Get a reference to the inner header map.","Get a mutable reference to the inner header map.","Include generated proto server and client items.","","","","Extract the message this response contains.","","Convert this to a hRPC request.","","Convert this to a hRPC response.","","","","","Create a new hRPC response.","Create a new request with the specified message.","Common server types and functions.","","","","","","","","","","","","","","","A boxed <code>Body</code> trait object.","A <code>hyper::Body</code>, mainly used for <code>Request</code> types.","Convert a <code>http_body::Body</code> into a <code>BoxBody</code>.","Create an empty <code>BoxBody</code>.","Create a “full” (single chunk) <code>BoxBody</code> containing the …","Generic client implementation with common methods.","A <code>hyper</code> HTTP client that supports HTTPS.","","","","","Connect a socket with the server and return it.","Connect a socket with the server, send a message and …","Error types.","Executes a unary request and returns the decoded response.","","","Creates a new HttpClient that you can use.","","","Set the function to modify request headers with before …","Creates a new client.","Creates a new client using the provided <code>HttpClient</code>.","hRPC socket used for streaming RPCs.","","","","","","Trying to work with already closed connection.","When reading: buffer capacity exhausted.When writing: …","Errors that can occur within <code>Client</code> operation.","Convenience type for <code>Client</code> operation result.","WebSocket connection closed normally. This informs you of …","Occurs if the data server responded with is not supported …","Occurs if an endpoint returns an error.","Occurs if request creation fails.","Occurs if hyper, the HTTP client, returns an error.","HTTP error.","A generic “error” for HTTP connections","HTTP format error.","Represents errors that can occur handling HTTP streams.","Occurs if URL scheme isn’t <code>http</code> or <code>https</code>.","Occurs if the given URL is invalid.","Errors that can occur while parsing the URL given to …","Occurs if an IO error is returned.","Input-output error. Apart from WouldBlock, these are …","The error type for I/O operations of the <code>Read</code>, <code>Write</code>, <code>Seek</code>…","Occurs if the data server responded with could not be …","Protocol violation.","Message send queue full.","Possible WebSocket errors.","Occurs if a websocket returns an error.","TLS error.","Invalid URL.","UTF coding error.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Converts a <code>NulError</code> into a <code>io::Error</code>.","","Converts an <code>ErrorKind</code> into an <code>Error</code>.","","","","","","","","","","","","","","","","","","","","Creates a new instance of an <code>Error</code> from a particular OS …","Returns a mutable reference to the inner error wrapped by …","Returns a reference to the inner error wrapped by this …","Return a reference to the lower level, inner error.","","","","","","Consumes the error, returning its cause.","Consumes the <code>Error</code>, returning its inner error (if any).","","","","","","","Return true if the underlying error has the same type as …","Returns true if the body write was aborted.","Returns true if this was about a <code>Request</code> that was …","Returns true if a sender’s channel is closed.","Returns true if this was an error from <code>Connect</code>.","Returns true if the connection closed before a message …","Returns true if this was an HTTP parse error.","Returns true if this was an HTTP parse error caused by an …","Returns true if this was an HTTP parse error caused by a …","Returns true if the error was caused by a timeout.","Returns true if this error was caused by user code.","Returns the corresponding <code>ErrorKind</code> for this error.","Returns an error representing the last OS error which …","Creates a new I/O error from a known kind of error as …","Returns the OS error that this error represents (if any).","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","A hRPC socket.","","","","","Close the socket.","","","","","Return whether the socket is closed or not.","Receive a message from the socket.","Send a message over the socket.","Spawns a parallel task that processes response messages …","Spawns a parallel task that processes a socket.","","","","","","","A boxed layer which takes hRPC <code>Handler</code>s and produces …","Type that contains a <code>Server</code> and implements …","Type that layers the handlers that are produced by a …","The core trait of <code>hrpc-rs</code> servers. This trait acts as a …","Type that contains two <code>Server</code>s and stacks (combines) them.","","","","","","","","","","","","","","Combines this server with another server.","Error types used by hRPC.","","","","Handler type and handlers used by hRPC.","","","","Turns this server into a type that implements <code>MakeService</code>…","","","","Layers this server with a layer that transforms handlers.","Creates a <code>Routes</code>, which will be used to build a …","","","","The router used by hRPC.","Serves this server. See <code>utils::serve</code> for more information.","Socket used by hRPC for “streaming” RPCs.","","","","","","","","","","","","","Other useful types, traits and functions used by hRPC.","","","","Trying to work with already closed connection.","When reading: buffer capacity exhausted.When writing: …","WebSocket connection closed normally. This informs you of …","Custom error that can be anything.","Trait that needs to be implemented to use an error type …","Occurs if a body of supported type could not be decoded.","HTTP error.","HTTP format error.","Input-output error. Apart from WouldBlock, these are …","Protocol violation.","Message send queue full.","A server error.","Shorthand type for `Result<T, ServerError>.","Possible WebSocket errors.","Occurs if a socket error occurs.","TLS error.","Invalid URL.","UTF coding error.","Create a response from this error.","Status code and message that will be used in client …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Convert this error into a response.","","","Creates a JSON error response from a message.","","","","","","","","","","","","","Call future used by <code>Handler</code>.","A hRPC handler.","","","","","","","Layer this handler.","Create a new handler from a <code>Service</code>.","A handler that responses to any request with not found.","","","","","","Builder type for inserting <code>Handler</code>s before building a …","Finalized <code>Routes</code>, ready for serving as a <code>Service</code>.","Set the service that will be used if no routes are …","","","","","Build the routes.","","Combine this with another <code>Routes</code>.","","","","","","","","Layer the routes that were added until this.","Create a new <code>Routes</code>.","","Add a new route.","","","","","","","","","A hRPC socket.","","","","","Close the socket.","","","","","Return whether the socket is closed or not.","Receive a message from the socket.","Send a message over the socket.","Spawns a parallel task that processes request messages …","Spawns a parallel task that processes a socket.","","","","","","Helper methods for working with <code>HeaderMap</code>.","Check if a header contains a string. Ignores casing for …","Check if a header is equal to a bytes array. Ignores …","Start serving a server on the specified address."],"i":[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,2,3,1,2,3,1,0,0,3,0,0,0,3,1,1,2,2,2,3,1,3,3,0,2,3,1,2,3,4,3,5,2,2,3,1,2,3,0,1,1,2,3,1,2,3,1,2,3,1,2,3,1,0,0,0,0,0,0,0,6,6,6,6,6,6,0,6,6,6,0,6,6,6,6,6,0,6,6,6,6,6,7,7,0,0,7,8,8,8,8,7,0,7,0,9,8,0,8,7,0,8,7,7,0,8,7,7,7,8,9,10,11,12,8,9,10,11,12,10,10,8,8,9,9,10,10,11,11,12,12,8,8,8,8,8,8,9,10,10,10,10,10,10,10,10,10,10,10,10,10,11,11,11,11,11,11,11,11,12,10,10,10,11,8,9,10,11,12,12,10,10,8,9,10,11,12,11,12,12,12,12,12,12,12,12,12,12,10,10,10,10,8,10,11,12,8,9,10,11,12,8,9,10,11,12,8,9,10,11,12,8,9,10,11,12,8,9,10,11,12,13,13,13,0,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,14,0,0,0,0,0,0,15,16,17,15,16,17,17,15,16,17,15,16,17,18,0,15,16,17,0,15,16,17,18,15,16,17,18,18,15,16,17,0,18,0,15,16,17,15,16,17,15,16,17,15,16,17,0,15,16,17,7,7,7,19,0,19,7,7,7,7,7,0,0,0,19,7,7,7,20,20,19,7,19,7,19,19,7,7,19,19,19,19,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,19,7,7,19,19,7,0,19,7,19,7,19,7,19,7,19,7,19,7,0,0,21,21,21,21,21,21,21,21,0,21,21,21,21,21,0,0,22,22,23,22,23,22,23,22,22,22,23,22,23,22,23,22,22,23,22,22,23,22,23,22,23,22,23,0,24,24,24,24,24,24,24,24,24,24,24,24,24,24,24,24,24,24,24,0,25,25,0],"f":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],null,null,[[],["request",3]],[[],["bytesmut",3]],[[["bytesmut",3]]],null,[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[],["headermap",3]],[[],["headermap",3]],null,[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[]],null,[[],[["option",4,["stderror"]],["stderror",8]]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],null,null,[[],["boxbody",6]],[[],["boxbody",6]],[[["bytes",3]],["boxbody",6]],null,null,[[]],[[]],[[],["client",3]],[[]],[[["request",3],["str",15]]],[[["str",15],["request",3]]],null,[[["str",15],["message",8],["request",3]]],[[["formatter",3]],["result",6]],[[]],[[],["httpclient",6]],[[]],[[],["response",3]],[[["fn",8],["arc",3,["fn"]]]],[[["uri",3]],["clientresult",6]],[[["httpclient",6],["uri",3]],["clientresult",6]],null,[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],[["option",4,["error"]],["error",8]]],[[],["str",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[]],[[["bodyorioerror",4,["error"]],["error",3]]],[[["ioerror",3]]],[[["socketerror",4]]],[[["decodebodyerror",4]]],[[["error",3]]],[[]],[[["nulerror",3]],["error",3]],[[["intoinnererror",3]],["error",3]],[[["errorkind",4]],["error",3]],[[["decodeerror",3]],["error",3]],[[["error",3]],["error",3]],[[["decompresserror",3]],["error",3]],[[["compresserror",3]],["error",3]],[[["elapsed",3]],["error",3]],[[["joinerror",3]],["error",3]],[[["error",3]],["error",3]],[[["error",3]],["error",3]],[[]],[[["encodeerror",3]],["error",3]],[[["invalidheadervalue",3]],["error",3]],[[["invalidmethod",3]],["error",3]],[[["invaliduriparts",3]],["error",3]],[[["invaliduri",3]],["error",3]],[[]],[[["invalidheadername",3]],["error",3]],[[["invalidstatuscode",3]],["error",3]],[[["infallible",4]],["error",3]],[[]],[[["i32",15]],["error",3]],[[],[["option",4,["error"]],["error",8]]],[[],[["option",4,["error"]],["error",8]]],[[],["error",8]],[[]],[[]],[[]],[[]],[[]],[[],[["option",4,["box"]],["box",3,["error","global"]]]],[[],[["option",4,["box"]],["box",3,["error","global"]]]],[[],[["error",3],["option",4,["error"]]]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["errorkind",4]],[[],["error",3]],[[["errorkind",4]],["error",3]],[[],[["option",4,["i32"]],["i32",15]]],[[],[["option",4,["stderror"]],["stderror",8]]],[[],[["option",4,["error"]],["error",8]]],[[],[["option",4,["error"]],["error",8]]],[[],[["option",4,["error"]],["error",8]]],[[],["string",3]],[[],["string",3]],[[],["string",3]],[[],["string",3]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],[[]],null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[]],[[]],[[],["response",3]],[[],["bool",15]],[[]],[[]],[[],[["result",4,["clienterror"]],["joinhandle",3,["result"]]]],[[],[["result",4,["clienterror"]],["joinhandle",3,["result"]]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["serverstack",3]],null,[[]],[[]],[[]],null,[[]],[[]],[[]],[[],["intomakeservice",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["layeredserver",3]],[[],["routes",3]],[[],["routes",3]],[[],["routes",3]],[[["context",3]],[["result",4],["poll",4,["result"]]]],null,[[],[["pin",3,["box"]],["box",3,["future"]]]],null,[[]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],null,[[]],[[]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],["httpresponse",6]],[[]],[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["formatter",3]],[["error",3],["result",4,["error"]]]],[[["socketerror",4]]],[[]],[[]],[[["decodebodyerror",4]]],[[["fromutf8error",3]],["error",4]],[[]],[[["invalidheadervalue",3]],["error",4]],[[["error",4]],["error",4]],[[["utf8error",3]],["error",4]],[[["urlerror",4]],["error",4]],[[["protocolerror",4]],["error",4]],[[["tostrerror",3]],["error",4]],[[["invalidheadername",3]],["error",4]],[[["error",3]],["error",4]],[[["invaliduri",3]],["error",4]],[[["error",3]],["error",4]],[[["capacityerror",4]],["error",4]],[[["tlserror",4]],["error",4]],[[["invalidstatuscode",3]],["error",4]],[[]],[[]],[[],[["option",4,["error"]],["error",4]]],[[],["httpresponse",6]],[[],["response",3]],[[],["response",3]],[[],[["vec",3,["u8"]],["u8",15]]],[[],[["option",4,["stderror"]],["stderror",8]]],[[],[["option",4,["error"]],["error",8]]],[[],["string",3]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],null,null,[[]],[[]],[[["httprequest",6]]],[[]],[[]],[[],["response",3]],[[]],[[]],[[],["handler",3]],[[["context",3]],[["result",4],["poll",4,["result"]]]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,[[]],[[]],[[]],[[]],[[]],[[],["routesfinalized",3]],[[["httprequest",6]]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["response",3]],[[],["response",3]],[[]],[[]],[[["context",3]],[["result",4],["poll",4,["result"]]]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],null,[[]],[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[]],[[]],[[],["response",3]],[[],["bool",15]],[[]],[[]],[[],[["joinhandle",3,["result"]],["result",4,["servererror"]]]],[[],[["result",4,["servererror"]],["joinhandle",3,["result"]]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,[[["headername",3],["str",15]],["bool",15]],[[["headername",3]],["bool",15]],[[]]],"p":[[4,"DecodeBodyError"],[3,"Response"],[3,"Request"],[8,"IntoRequest"],[8,"IntoResponse"],[3,"Client"],[4,"SocketError"],[4,"ClientError"],[4,"InvalidUrlKind"],[3,"IoError"],[3,"HttpError"],[3,"HyperError"],[13,"EndpointError"],[3,"Socket"],[3,"LayeredServer"],[3,"ServerStack"],[3,"IntoMakeService"],[8,"Server"],[4,"ServerError"],[8,"CustomError"],[3,"Handler"],[3,"Routes"],[3,"RoutesFinalized"],[3,"Socket"],[8,"HeaderMapExt"]]},\
"hrpc_build":{"doc":"","t":[3,16,16,8,16,8,11,11,11,11,0,10,11,11,10,10,11,5,11,5,11,11,5,11,11,11,10,10,11,10,10,10,10,11,10,11,10,0,10,11,11,11,11,11,11,5,5],"n":["Builder","Comment","Comment","Method","Method","Service","borrow","borrow_mut","build_client","build_server","client","client_streaming","clone","clone_into","comment","comment","compile","compile_protos","compile_with_config","configure","extern_path","field_attribute","fmt","fmt","format","from","identifier","identifier","into","methods","name","name","options","out_dir","package","proto_path","request_response_name","server","server_streaming","to_owned","try_from","try_into","type_attribute","type_id","vzip","generate","generate"],"q":["hrpc_build","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc_build::client","hrpc_build::server"],"d":["Service generator builder.","Comment type.","Comment type.","Method generation trait.","Method type.","Service generation trait.","","","Enable or disable hRPC client code generation.","Enable or disable hRPC server code generation.","Service code generation for client","Method is streamed by client.","","","Get comments about this item.","Get comments about this item.","Compile the .proto files and execute code generation.","Simple <code>.proto</code> compiling. Use <code>configure</code> instead if you …","Compile the .proto files and execute code generation …","Configure <code>hrpc-build</code> code generation.","Declare externally provided Protobuf package or type.","Add additional attribute to matched messages, enums, and …","Format files under the out_dir with rustfmt","","Enable the output to be formated by rustfmt.","","Identifier used to generate type name.","Identifier used to generate type name.","","Methods provided by service.","Name of service.","Name of method.","Get options of this item.","Set the output directory to generate code to.","Package name of service.","Set the path to where tonic will search for the …","Type name of request and response.","Service code generation for server","Method is streamed by server.","","","","Add additional attribute to matched messages, enums, and …","","","Generate service for client.","Generate service for Server."],"i":[0,1,2,0,1,0,3,3,3,3,0,2,3,3,1,2,3,0,3,0,3,3,0,3,3,3,1,2,3,1,1,2,2,3,1,3,2,0,2,3,3,3,3,3,3,0,0],"f":[null,null,null,null,null,null,[[]],[[]],[[["bool",15]]],[[["bool",15]]],null,[[],["bool",15]],[[],["builder",3]],[[]],[[]],[[]],[[],["result",6]],[[],["result",6]],[[["config",3]],["result",6]],[[],["builder",3]],[[]],[[["str",15],["asref",8,["str"]]]],[[["str",15]]],[[["formatter",3]],["result",6]],[[["bool",15]]],[[]],[[],["str",15]],[[],["str",15]],[[]],[[]],[[],["str",15]],[[],["str",15]],[[],["vec",3]],[[]],[[],["str",15]],[[]],[[["str",15]]],null,[[],["bool",15]],[[]],[[],["result",4]],[[],["result",4]],[[["str",15],["asref",8,["str"]]]],[[],["typeid",3]],[[]],[[["str",15]],["tokenstream",3]],[[["str",15]],["tokenstream",3]]],"p":[[8,"Service"],[8,"Method"],[3,"Builder"]]},\
"interop":{"doc":"","t":[3,3,13,3,4,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,12,12,0,11,11,11,11,0,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,8,3,11,11,11,11,11,11,12,11,11,11,10,11,10,11,11,10,11,11,11,11,11,11,11,11],"n":["MuService","Ping","PingEmpty","Pong","ServerError","as_status_message","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clear","clear","client","clone","clone","clone","clone_into","clone_into","clone_into","default","default","encode_raw","encode_raw","encoded_len","encoded_len","eq","eq","fmt","fmt","fmt","fmt","fmt","from","from","from","from","into","into","into","into","into_request","into_request","into_response","into_response","into_response","into_response","main","merge_field","merge_field","mu","mu","mu","mu_client","mu_mu","mu_mute","mu_mute_middleware","mu_mute_on_upgrade","mu_server","ne","ne","server","to_owned","to_owned","to_owned","to_string","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","MuClient","borrow","borrow_mut","clone","clone_into","connect","fmt","from","inner","inner","into","into_response","mu","mu_mu","mu_mute","new_http","new_inner","to_owned","try_from","try_into","type_id","vzip","Mu","MuServer","borrow","borrow_mut","clone","clone_into","fmt","from","inner","into","into_response","make_routes","mu","mu_middleware","mu_mu","mu_mu_middleware","mu_mu_on_upgrade","mu_mute","mu_mute_middleware","mu_mute_on_upgrade","new","to_owned","try_from","try_into","type_id","vzip"],"q":["interop","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","interop::mu_client","","","","","","","","","","","","","","","","","","","","","","interop::mu_server","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["","Ping message.","","Pong message.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Generated client implementations.","","","","","Generated server implementations.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Mu RPC.","MuMu RPC. ``` test ```","MuMute RPC.","","","","","","","","Generated trait containing hRPC methods that should be …","","","","","","","","","","","","Mu RPC.","Filter to be run before all API operations but after API …","MuMu RPC. ``` test ```","Filter to be run before all API operations but after API …","Method that can be used to modify the response sent when …","MuMute RPC.","Filter to be run before all API operations but after API …","Method that can be used to modify the response sent when …","Create a new service server.","","","","",""],"i":[0,0,1,0,0,1,2,3,4,1,2,3,4,1,2,3,0,2,3,4,2,3,4,2,3,2,3,2,3,2,3,2,3,4,1,1,2,3,4,1,2,3,4,1,2,3,2,3,4,1,0,2,3,4,2,3,0,4,4,4,4,0,2,3,0,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,0,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,0,0,6,6,6,6,6,6,6,6,6,6,7,7,7,7,7,7,7,7,6,6,6,6,6,6],"f":[null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["ping",3]],[[],["pong",3]],[[],["muservice",3]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["usize",15]],[[],["usize",15]],[[["ping",3]],["bool",15]],[[["pong",3]],["bool",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[],["response",3]],[[]],[[["decodecontext",3],["u32",15],["wiretype",4]],[["decodeerror",3],["result",4,["decodeerror"]]]],[[["decodecontext",3],["u32",15],["wiretype",4]],[["decodeerror",3],["result",4,["decodeerror"]]]],[[["request",3,["ping"]],["ping",3]],[["pin",3,["box"]],["box",3,["future"]]]],null,null,null,[[["socket",3,["ping","pong"]],["pong",3],["ping",3],["request",3]],[["box",3,["future"]],["pin",3,["box"]]]],[[["socket",3,["ping","pong"]],["pong",3],["ping",3],["request",3]],[["box",3,["future"]],["pin",3,["box"]]]],[[["str",15]],["hrpclayer",6]],[[["httpresponse",6]],["httpresponse",6]],null,[[["ping",3]],["bool",15]],[[["pong",3]],["bool",15]],[[]],[[]],[[]],[[]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],null,[[]],[[]],[[],["muclient",3]],[[]],[[],["clientresult",6]],[[["formatter",3]],["result",6]],[[]],[[],["client",3]],null,[[]],[[],["response",3]],[[]],[[]],[[]],[[["httpclient",6],["uri",3]],["clientresult",6]],[[["client",3]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[]],null,[[]],[[],["response",3]],[[],["routes",3]],[[["hrpcrequest",3,["ping"]],["ping",3]],[["pin",3,["box"]],["box",3,["future"]]]],[[["str",15]],["hrpclayer",6]],[[["ping",3],["hrpcrequest",3],["socket",3,["ping","pong"]],["pong",3]],[["box",3,["future"]],["pin",3,["box"]]]],[[["str",15]],["hrpclayer",6]],[[["httpresponse",6]],["httpresponse",6]],[[["ping",3],["hrpcrequest",3],["socket",3,["ping","pong"]],["pong",3]],[["box",3,["future"]],["pin",3,["box"]]]],[[["str",15]],["hrpclayer",6]],[[["httpresponse",6]],["httpresponse",6]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]]],"p":[[4,"ServerError"],[3,"Ping"],[3,"Pong"],[3,"MuService"],[3,"MuClient"],[3,"MuServer"],[8,"Mu"]]},\
"server":{"doc":"","t":[3,11,11,11,11,11,11,11,5,11,11,11,11,11,11],"n":["GreeterService","borrow","borrow_mut","clone","clone_into","from","into","into_response","main","to_owned","try_from","try_into","type_id","vzip","welcome_user"],"q":["server","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","","","","",""],"i":[0,1,1,1,1,1,1,1,0,1,1,1,1,1,1],"f":[null,[[]],[[]],[[],["greeterservice",3]],[[]],[[]],[[]],[[],["response",3]],[[],[["boxerror",6],["result",4,["boxerror"]]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],[[["request",3,["welcomeuserrequest"]],["welcomeuserrequest",3]],[["pin",3,["box"]],["box",3,["future"]]]]],"p":[[3,"GreeterService"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};