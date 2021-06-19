var searchIndex = JSON.parse('{\
"hrpc":{"doc":"Common code used in hRPC code generation.","t":[8,3,23,11,11,0,11,11,11,11,11,11,11,11,11,11,14,11,11,10,11,11,11,11,11,14,14,14,14,0,11,11,11,11,11,3,4,6,13,13,13,4,13,13,13,13,13,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,18,18,18,18,18,18,18,13,8,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,18,13,18,18,18,18,18,18,18,18,18,18,13,18,18,18,18,18,18,18,18,18,18,18,18,3,3,18,18,18,3,3,4,3,18,18,18,18,18,18,18,18,18,18,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,10,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,11,11,11,11,11,11,11],"n":["IntoRequest","Request","async_trait","borrow","borrow_mut","client","clone","clone_into","empty","fmt","from","from_parts","get_header","get_header_map","get_message","header","include_proto","into","into_parts","into_request","into_request","into_request","map","message","new","return_closed","return_print","serve_multiple","serve_multiple_tls","server","to_owned","try_from","try_into","type_id","vzip","Client","ClientError","ClientResult","EndpointError","InvalidScheme","InvalidUrl","InvalidUrlKind","Io","MessageDecode","NonProtobuf","Reqwest","SocketError","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","clone","clone_into","connect_socket","connect_socket_req","execute_request","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","into","into","into","into_request","into_request","into_request","new","socket","source","to_owned","to_string","to_string","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","vzip","vzip","vzip","endpoint","raw_error","status","ReadSocket","Socket","borrow","borrow","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","close","close","fmt","fmt","from","from","get_message","get_message","into","into","into_request","into_request","read_only","send_message","to_owned","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip","ACCEPTED","ALREADY_REPORTED","BAD_GATEWAY","BAD_REQUEST","CONFLICT","CONTINUE","CREATED","ClosedNormally","CustomError","DECODE_ERROR","EXPECTATION_FAILED","FAILED_DEPENDENCY","FORBIDDEN","FOUND","GATEWAY_TIMEOUT","GONE","HTTP_VERSION_NOT_SUPPORTED","IM_A_TEAPOT","IM_USED","INSUFFICIENT_STORAGE","INTERNAL_SERVER_ERROR","INTERNAL_SERVER_ERROR","LENGTH_REQUIRED","LOCKED","LOOP_DETECTED","METHOD_NOT_ALLOWED","METHOD_NOT_ALLOWED","MISDIRECTED_REQUEST","MOVED_PERMANENTLY","MULTIPLE_CHOICES","MULTI_STATUS","MessageDecode","NETWORK_AUTHENTICATION_REQUIRED","NON_AUTHORITATIVE_INFORMATION","NOT_ACCEPTABLE","NOT_EXTENDED","NOT_FOUND","NOT_FOUND_ERROR","NOT_IMPLEMENTED","NOT_MODIFIED","NO_CONTENT","OK","Other","PARTIAL_CONTENT","PAYLOAD_TOO_LARGE","PAYMENT_REQUIRED","PERMANENT_REDIRECT","PRECONDITION_FAILED","PRECONDITION_REQUIRED","PROCESSING","PROXY_AUTHENTICATION_REQUIRED","RANGE_NOT_SATISFIABLE","REQUEST_HEADER_FIELDS_TOO_LARGE","REQUEST_TIMEOUT","RESET_CONTENT","ReadSocket","ReadSocketArc","SEE_OTHER","SERVICE_UNAVAILABLE","SWITCHING_PROTOCOLS","Socket","SocketArc","SocketError","StatusCode","TEMPORARY_REDIRECT","TOO_MANY_REQUESTS","UNAUTHORIZED","UNAVAILABLE_FOR_LEGAL_REASONS","UNPROCESSABLE_ENTITY","UNSUPPORTED_MEDIA_TYPE","UPGRADE_REQUIRED","URI_TOO_LONG","USE_PROXY","VARIANT_ALSO_NEGOTIATES","WriteSocket","WriteSocketArc","as_str","as_u16","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","canonical_reason","clonable","clonable","clonable","clone","clone","clone","clone","clone_into","clone_into","clone_into","clone_into","cmp","code","combine","combine","default","eq","eq","equivalent","filters","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from_bytes","from_str","from_u16","get_hash","hash","into","into","into","into","into","into","into","into","into_request","into_request","into_request","into_request","into_request","into_request","into_request","into_request","into_response","is_client_error","is_informational","is_redirection","is_server_error","is_success","json_err_bytes","message","ne","new","new","new","partial_cmp","receive_message","receive_message","receive_message","receive_message","send_message","send_message","send_message","send_message","source","split","split","to_owned","to_owned","to_owned","to_owned","to_string","to_string","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","vzip","vzip","vzip","vzip","rate","Rate","State","borrow","borrow","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","fmt","fmt","from","from","into","into","into_request","into_request","new","new","rate_limit","to_owned","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip"],"q":["hrpc","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::client","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::client::ClientError","","","hrpc::client::socket","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::server","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc::server::filters","hrpc::server::filters::rate","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["Trait used for blanket impls on generated protobuf types.","A hRPC request.","","","","Common client types and functions.","","","Create an empty request.","","","Create a request from parts.","Get a header.","Get a reference to the inner header map.","Get a reference to the inner message.","Change / add a header.","Include generated proto server and client items.","","Destructure this request into parts.","Convert this to a request.","","","Map the contained message.","Change the contained message.","Create a new request with the specified message.","Return if the socket is closed normally, otherwise return …","Return if the socket is closed normally, otherwise print …","Serves multiple services’ filters on the same address.","Serves multiple services’ filters on the same address. …","Common server types and functions.","","","","","","Generic client implementation with common methods.","Errors that can occur within <code>Client</code> operation.","Convenience type for <code>Client</code> operation result.","Occurs if an endpoint returns an error.","Occurs if URL scheme isn’t <code>http</code> or <code>https</code>.","Occurs if the given URL is invalid.","Errors that can occur while parsing the URL given to …","Occurs if an IO error is returned.","Occurs if the data server responded with can’t be …","Occurs if the data server responded with isn’t a …","Occurs if reqwest, the HTTP client, returns an error.","Occurs if a websocket returns an error.","","","","","","","","","Connect a socket with the server and return it.","Connect a socket with the server, send a message and …","Executes an unary request returns the decoded response.","","","","","","","","","","","","","","","","","","","Creates a new client.","Socket implementations.","","","","","","","","","","","","","","","","","","","","A read-only version of [<code>Socket</code>].","A websocket, wrapped for ease of use with protobuf …","","","","","","","","","Close this websocket. All subsequent message sending …","Close this websocket.","","","","","Get a message from the websocket.","Get a message from the websocket.","","","","","Converts this socket to a read-only socket.","Send a protobuf message over the websocket.","","","","","","","","","","","202 Accepted [RFC7231, Section 6.3.3]","208 Already Reported [RFC5842]","502 Bad Gateway [RFC7231, Section 6.6.3]","400 Bad Request [RFC7231, Section 6.5.1]","409 Conflict [RFC7231, Section 6.5.8]","100 Continue [RFC7231, Section 6.2.1]","201 Created [RFC7231, Section 6.3.2]","The socket is closed normally. This is NOT an error.","Trait that needs to be implemented to use an error type …","Status code and error body used to respond when a …","417 Expectation Failed [RFC7231, Section 6.5.14]","424 Failed Dependency [RFC4918]","403 Forbidden [RFC7231, Section 6.5.3]","302 Found [RFC7231, Section 6.4.3]","504 Gateway Timeout [RFC7231, Section 6.6.5]","410 Gone [RFC7231, Section 6.5.9]","505 HTTP Version Not Supported [RFC7231, Section 6.6.6]","418 I’m a teapot [curiously not registered by IANA but …","226 IM Used [RFC3229]","507 Insufficient Storage [RFC4918]","Status code and error body used to respond when an …","500 Internal Server Error [RFC7231, Section 6.6.1]","411 Length Required [RFC7231, Section 6.5.10]","423 Locked [RFC4918]","508 Loop Detected [RFC5842]","Status code and error body used to respond when a method …","405 Method Not Allowed [RFC7231, Section 6.5.5]","421 Misdirected Request RFC7540, Section 9.1.2","301 Moved Permanently [RFC7231, Section 6.4.2]","300 Multiple Choices [RFC7231, Section 6.4.1]","207 Multi-Status [RFC4918]","Error occured while decoding protobuf data.","511 Network Authentication Required [RFC6585]","203 Non-Authoritative Information [RFC7231, Section 6.3.4]","406 Not Acceptable [RFC7231, Section 6.5.6]","510 Not Extended [RFC2774]","404 Not Found [RFC7231, Section 6.5.4]","Status code and error body used to respond when a not …","501 Not Implemented [RFC7231, Section 6.6.2]","304 Not Modified [RFC7232, Section 4.1]","204 No Content [RFC7231, Section 6.3.5]","200 OK [RFC7231, Section 6.3.1]","Some error occured in socket.","206 Partial Content [RFC7233, Section 4.1]","413 Payload Too Large [RFC7231, Section 6.5.11]","402 Payment Required [RFC7231, Section 6.5.2]","308 Permanent Redirect [RFC7238]","412 Precondition Failed [RFC7232, Section 4.2]","428 Precondition Required [RFC6585]","102 Processing [RFC2518]","407 Proxy Authentication Required [RFC7235, Section 3.2]","416 Range Not Satisfiable [RFC7233, Section 4.4]","431 Request Header Fields Too Large [RFC6585]","408 Request Timeout [RFC7231, Section 6.5.7]","205 Reset Content [RFC7231, Section 6.3.6]","A read only socket.","A clonable read only socket.","303 See Other [RFC7231, Section 6.4.4]","503 Service Unavailable [RFC7231, Section 6.6.4]","101 Switching Protocols [RFC7231, Section 6.2.2]","A web socket.","A cloneable web socket.","","An HTTP status code (<code>status-code</code> in RFC 7230 et al.).","307 Temporary Redirect [RFC7231, Section 6.4.7]","429 Too Many Requests [RFC6585]","401 Unauthorized [RFC7235, Section 3.1]","451 Unavailable For Legal Reasons [RFC7725]","422 Unprocessable Entity [RFC4918]","415 Unsupported Media Type [RFC7231, Section 6.5.13]","426 Upgrade Required [RFC7231, Section 6.5.15]","414 URI Too Long [RFC7231, Section 6.5.12]","305 Use Proxy [RFC7231, Section 6.4.5]","506 Variant Also Negotiates [RFC2295]","A write only socket.","A clonable write only socket.","Returns a &str representation of the <code>StatusCode</code>","Returns the <code>u16</code> corresponding to this <code>StatusCode</code>.","","","","","","","","","","","","","","","","","Get the standardised <code>reason-phrase</code> for this status code.","Make a cloneable and shared version of this socket.","Create a clonable and shared version of ReadSocket.","Create a clonable and shared version of WriteSocket.","","","","","","","","","","Status code that will be used in client response.","Combine read and write sockets into one.","Combine read and write sockets into one.","","","","","Useful filters.","","","","","","","","","","","","","","","","","","","","Converts a &[u8] to a status code","","Converts a u16 to a status code.","","","","","","","","","","","","","","","","","","","","Check if status is within 400-499.","Check if status is within 100-199.","Check if status is within 300-399.","Check if status is within 500-599.","Check if status is within 200-299.","Creates a JSON error response from a message.","Message that will be used in client response.","","","","","","Receive a message from the socket.","Receive a message from the socket.","Receive a message from the socket.","Receive a message from the socket.","Send a message over the socket.","Send a message over the socket.","Send a message over the socket.","Send a message over the socket.","","Split this socket into read and write counterparts.","Split this socket into read and write counterparts.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","A rate of requests per time period.","","","","","","","","","","","","","","","","","","Create a new rate.","","Creates a filter that will return an <code>Err(error)</code> with <code>error</code>…","","","","","","","","","",""],"i":[0,0,0,1,1,0,1,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1,0,0,0,0,0,1,1,1,1,1,0,0,0,3,4,3,0,3,3,3,3,3,5,3,4,5,3,4,5,5,5,5,5,5,3,3,4,4,5,3,3,3,3,3,4,5,3,4,5,3,4,5,0,3,5,3,4,5,3,4,5,3,4,5,3,4,5,3,4,6,6,6,0,0,7,8,7,8,7,8,7,8,7,8,7,8,7,8,7,8,7,8,7,8,7,7,7,8,7,8,7,8,7,8,7,8,9,9,9,9,9,9,9,10,0,11,9,9,9,9,9,9,9,9,9,9,11,9,9,9,9,11,9,9,9,9,9,10,9,9,9,9,9,11,9,9,9,9,10,9,9,9,9,9,9,9,9,9,9,9,9,0,0,9,9,9,0,0,0,0,9,9,9,9,9,9,9,9,9,9,0,0,9,9,12,13,14,15,16,17,10,9,12,13,14,15,16,17,10,9,9,13,15,17,12,14,16,9,12,14,16,9,9,11,12,13,9,9,9,9,0,12,13,14,15,16,17,10,10,9,9,12,13,14,15,16,17,10,9,9,9,9,9,9,9,12,13,14,15,16,17,10,9,12,13,14,15,16,17,10,9,9,9,9,9,9,9,0,11,9,13,15,17,9,12,13,14,15,12,13,16,17,10,12,13,12,14,16,9,10,9,12,13,14,15,16,17,10,9,9,9,9,12,13,14,15,16,17,10,9,12,13,14,15,16,17,10,9,12,13,14,15,16,17,10,9,0,0,0,18,19,18,19,18,19,18,19,18,19,18,19,18,19,18,19,18,19,0,18,19,18,19,18,19,18,19,18,19],"f":[null,null,null,[[]],[[]],null,[[],["request",3]],[[]],[[],["request",3]],[[["formatter",3]],["result",6]],[[]],[[]],[[["headername",3]],[["headervalue",3],["option",4]]],[[],["headermap",3]],[[]],[[["headervalue",3],["headername",3]]],null,[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[["fnonce",8]],["request",3]],[[],["request",3]],[[]],null,null,null,null,null,[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[],["client",3]],[[]],[[["request",3],["str",15]]],[[["str",15],["request",3]]],[[["message",8],["str",15],["request",3]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[["decodeerror",3]]],[[["error",3]]],[[["error",4]]],[[["error",3]]],[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[["url",3],["client",3]],["clientresult",6]],null,[[],[["stderror",8],["option",4]]],[[]],[[],["string",3]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[],["readsocket",3]],[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["readsocket",3]],[[]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],["str",15]],[[],["u16",15]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],[["str",15],["option",4]]],[[],["socketarc",3]],[[],["readsocketarc",3]],[[],["writesocketarc",3]],[[],["socketarc",3]],[[],["readsocketarc",3]],[[]],[[],["statuscode",3]],[[]],[[]],[[]],[[]],[[["statuscode",3]],["ordering",4]],[[],["statuscode",3]],[[["readsocketarc",3],["writesocketarc",3]]],[[["readsocket",3],["writesocket",3]]],[[],["statuscode",3]],[[["u16",15]],["bool",15]],[[["statuscode",3]],["bool",15]],[[],["bool",15]],null,[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],[["error",3],["result",4]]],[[["formatter",3]],[["error",3],["result",4]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[["statuscode",3]],["statuscode",3]],[[],[["invalidstatuscode",3],["statuscode",3],["result",4]]],[[["str",15]],[["invalidstatuscode",3],["statuscode",3],["result",4]]],[[["u16",15]],[["invalidstatuscode",3],["statuscode",3],["result",4]]],[[],["u64",15]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],[["response",3],["body",3]]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[],["bool",15]],[[["str",15]],[["u8",15],["vec",3]]],[[],[["u8",15],["vec",3]]],[[["statuscode",3]],["bool",15]],[[["websocket",3]]],[[["websocket",3],["splitstream",3]]],[[["splitsink",3],["websocket",3],["wsmessage",3]]],[[["statuscode",3]],[["option",4],["ordering",4]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],[["stderror",8],["option",4]]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["string",3]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["u16",15]],[["statuscode",3],["result",4]]],[[["str",15]],[["statuscode",3],["result",4]]],[[],[["statuscode",3],["result",4]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],null,null,null,[[]],[[]],[[]],[[]],[[],["rate",3]],[[],["state",3]],[[]],[[]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[["duration",3],["u64",15]]],[[["rate",3]]],[[["rate",3]]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]]],"p":[[3,"Request"],[8,"IntoRequest"],[4,"ClientError"],[4,"InvalidUrlKind"],[3,"Client"],[13,"EndpointError"],[3,"Socket"],[3,"ReadSocket"],[3,"StatusCode"],[4,"SocketError"],[8,"CustomError"],[3,"SocketArc"],[3,"Socket"],[3,"ReadSocketArc"],[3,"ReadSocket"],[3,"WriteSocketArc"],[3,"WriteSocket"],[3,"Rate"],[3,"State"]]},\
"hrpc_build":{"doc":"","t":[3,16,16,8,16,8,11,11,11,11,0,10,11,11,10,10,11,5,11,5,11,11,5,11,11,11,10,10,11,10,10,10,10,11,10,11,10,0,10,11,11,11,11,11,11,5,5],"n":["Builder","Comment","Comment","Method","Method","Service","borrow","borrow_mut","build_client","build_server","client","client_streaming","clone","clone_into","comment","comment","compile","compile_protos","compile_with_config","configure","extern_path","field_attribute","fmt","fmt","format","from","identifier","identifier","into","methods","name","name","options","out_dir","package","proto_path","request_response_name","server","server_streaming","to_owned","try_from","try_into","type_attribute","type_id","vzip","generate","generate"],"q":["hrpc_build","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","hrpc_build::client","hrpc_build::server"],"d":["Service generator builder.","Comment type.","Comment type.","Method generation trait.","Method type.","Service generation trait.","","","Enable or disable hRPC client code generation.","Enable or disable hRPC server code generation.","Service code generation for client","Method is streamed by client.","","","Get comments about this item.","Get comments about this item.","Compile the .proto files and execute code generation.","Simple <code>.proto</code> compiling. Use [<code>configure</code>] instead if you …","Compile the .proto files and execute code generation …","Configure <code>hrpc-build</code> code generation.","Declare externally provided Protobuf package or type.","Add additional attribute to matched messages, enums, and …","Format files under the out_dir with rustfmt","","Enable the output to be formated by rustfmt.","","Identifier used to generate type name.","Identifier used to generate type name.","","Methods provided by service.","Name of service.","Name of method.","Get options of this item.","Set the output directory to generate code to.","Package name of service.","Set the path to where tonic will search for the …","Type name of request and response.","Service code generation for server","Method is streamed by server.","","","","Add additional attribute to matched messages, enums, and …","","","Generate service for client.","Generate service for Server."],"i":[0,1,2,0,1,0,3,3,3,3,0,2,3,3,1,2,3,0,3,0,3,3,0,3,3,3,1,2,3,1,1,2,2,3,1,3,2,0,2,3,3,3,3,3,3,0,0],"f":[null,null,null,null,null,null,[[]],[[]],[[["bool",15]]],[[["bool",15]]],null,[[],["bool",15]],[[],["builder",3]],[[]],[[]],[[]],[[],["result",6]],[[],["result",6]],[[["config",3]],["result",6]],[[],["builder",3]],[[]],[[["asref",8],["str",15]]],[[["str",15]]],[[["formatter",3]],["result",6]],[[["bool",15]]],[[]],[[],["str",15]],[[],["str",15]],[[]],[[]],[[],["str",15]],[[],["str",15]],[[],["vec",3]],[[]],[[],["str",15]],[[]],[[["str",15]]],null,[[],["bool",15]],[[]],[[],["result",4]],[[],["result",4]],[[["asref",8],["str",15]]],[[],["typeid",3]],[[]],[[["str",15]],["tokenstream",3]],[[["str",15]],["tokenstream",3]]],"p":[[8,"Service"],[8,"Method"],[3,"Builder"]]},\
"interop":{"doc":"","t":[3,13,3,3,4,13,11,11,11,11,11,11,11,11,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,11,11,11,12,12,0,11,11,11,11,11,11,0,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,11,11,11,11,11,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,16,8,16,16,3,11,11,11,11,11,11,11,12,11,11,10,10,11,11,10,10,11,11,10,11,11,11,11,11,11,11,11],"n":["Ping","PingEmpty","Pong","Server","ServerError","TooFast","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clear","clear","client","clone","clone","clone_into","clone_into","code","default","default","encode_raw","encode_raw","encoded_len","encoded_len","eq","eq","fmt","fmt","fmt","fmt","fmt","from","from","from","from","into","into","into","into","into_request","into_request","into_request","into_request","main","merge_field","merge_field","message","mu","mu","mu","mu_client","mu_mu","mu_mu_validation","mu_mute","mu_mute_on_upgrade","mu_mute_validation","mu_pre","mu_server","ne","ne","server","to_owned","to_owned","to_string","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","vzip","vzip","vzip","vzip","MuClient","borrow","borrow_mut","clone","clone_into","fmt","from","inner","inner","into","into_request","mu","mu_mu","mu_mute","new","new_inner","to_owned","try_from","try_into","type_id","vzip","Error","Mu","MuMuValidationType","MuMuteValidationType","MuServer","borrow","borrow_mut","clone","clone_into","filters","fmt","from","inner","into","into_request","mu","mu_mu","mu_mu_on_upgrade","mu_mu_pre","mu_mu_validation","mu_mute","mu_mute_on_upgrade","mu_mute_pre","mu_mute_validation","mu_pre","new","serve","to_owned","try_from","try_into","type_id","vzip"],"q":["interop","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","interop::mu_client","","","","","","","","","","","","","","","","","","","","","interop::mu_server","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Generated client implementations.","","","","","","","Generated server implementations.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Generated trait containing hRPC methods that should be …","","","","","","","","Convert this service to <code>warp</code> <code>Filter</code>s.","","","","","","","","","","","","","","","","Create a new service server.","Start serving.","","","","",""],"i":[0,1,0,0,0,1,2,3,4,1,2,3,4,1,2,3,0,2,3,2,3,1,2,3,2,3,2,3,2,3,2,3,4,1,1,2,3,4,1,2,3,4,1,2,3,4,1,0,2,3,1,4,2,3,0,4,4,4,4,4,4,0,2,3,0,2,3,1,2,3,4,1,2,3,4,1,2,3,4,1,2,3,4,1,0,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,6,0,6,6,0,7,7,7,7,7,7,7,7,7,7,6,6,6,6,6,6,6,6,6,6,7,7,7,7,7,7,7],"f":[null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["ping",3]],[[],["pong",3]],[[]],[[]],[[],["statuscode",3]],[[]],[[]],[[]],[[]],[[],["usize",15]],[[],["usize",15]],[[["ping",3]],["bool",15]],[[["pong",3]],["bool",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[],["request",3]],[[]],[[["wiretype",4],["u32",15],["decodecontext",3]],[["decodeerror",3],["result",4]]],[[["wiretype",4],["u32",15],["decodecontext",3]],[["decodeerror",3],["result",4]]],[[],[["u8",15],["vec",3]]],[[["request",3],["ping",3]],[["box",3],["pin",3]]],null,null,null,[[["writesocket",3],["pong",3]],[["pin",3],["box",3]]],[[["request",3],["option",4]],[["pin",3],["box",3]]],[[["socket",3],["ping",3],["pong",3]],[["pin",3],["box",3]]],[[["response",6]],["response",6]],[[["request",3]],[["box",3],["pin",3]]],[[],["boxedfilter",3]],null,[[["ping",3]],["bool",15]],[[["pong",3]],["bool",15]],[[]],[[]],[[]],[[],["string",3]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[]],null,[[]],[[]],[[],["muclient",3]],[[]],[[["formatter",3]],["result",6]],[[]],[[],["client",3]],null,[[]],[[],["request",3]],[[]],[[]],[[]],[[["reqwestclient",3],["url",3]],["clientresult",6]],[[["client",3]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]],null,null,null,null,null,[[]],[[]],[[],["muserver",3]],[[]],[[],["boxedfilter",3]],[[["formatter",3]],["result",6]],[[]],null,[[]],[[],["request",3]],[[["request",3],["ping",3]],[["pin",3],["box",3]]],[[["pong",3],["writesocket",3]],[["pin",3],["box",3]]],[[["response",6]],["response",6]],[[],["boxedfilter",3]],[[["request",3],["option",4]],[["pin",3],["box",3]]],[[["pong",3],["socket",3],["ping",3]],[["pin",3],["box",3]]],[[["response",6]],["response",6]],[[],["boxedfilter",3]],[[["request",3]],[["box",3],["pin",3]]],[[],["boxedfilter",3]],[[]],[[["into",8],["socketaddr",4]]],[[]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[]]],"p":[[4,"ServerError"],[3,"Ping"],[3,"Pong"],[3,"Server"],[3,"MuClient"],[8,"Mu"],[3,"MuServer"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};