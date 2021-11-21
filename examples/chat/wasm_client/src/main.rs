use std::cell::RefCell;

use web_sys::HtmlInputElement as InputElement;
use yew::prelude::*;

use chat_common::{
    chat::{chat_client::*, *},
    *,
};
use hrpc::{
    client::{
        error::{ClientError, ClientResult as HrpcClientResult},
        socket::Socket,
        transport::http::{Wasm, WasmError},
    },
    exports::futures_util::FutureExt,
    proto::Error as HrpcError,
    Response,
};

type ClientResult<T> = HrpcClientResult<T, ClientError<WasmError>>;

enum Msg {
    SendMessage(String),
    SendMessageResult(ClientResult<Response<Empty>>),
    PollMessagesResult {
        socket: Socket<Empty, Message>,
        res: Result<Message, HrpcError>,
    },
    SocketCreateResult(ClientResult<Socket<Empty, Message>>),
    Nothing,
}

struct Model {
    messages: Vec<String>,
    last_error: String,
    current_message: String,
}

#[derive(Clone)]
struct Props {
    client: RefCell<ChatClient<Wasm>>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        // There is no notion of "equality" for client provided by hRPC
        // and since we won't change the server URL we can just they are
        // always equal
        true
    }
}

impl Properties for Props {
    type Builder = ();

    fn builder() -> Self::Builder {}
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Model>) -> Self {
        Self {
            messages: Vec::new(),
            last_error: String::new(),
            current_message: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Model>, msg: Self::Message) -> bool {
        match msg {
            Msg::SendMessage(content) => {
                let fut = ctx
                    .props()
                    .client
                    .borrow_mut()
                    .send_message(Message { content })
                    .map(Msg::SendMessageResult);
                ctx.link().send_future(fut);
                false
            }
            Msg::SendMessageResult(res) => match res {
                Ok(_) => false,
                Err(err) => {
                    self.last_error = err.to_string();
                    true
                }
            },
            Msg::SocketCreateResult(res) => match res {
                Ok(mut socket) => {
                    let fut = async move {
                        let res = socket.receive_message().await;
                        Msg::PollMessagesResult { socket, res }
                    };
                    ctx.link().send_future(fut);
                    false
                }
                Err(err) => {
                    self.last_error = err.to_string();
                    true
                }
            },
            Msg::PollMessagesResult { mut socket, res } => {
                match res {
                    Ok(msg) => self.messages.push(msg.content),
                    Err(err) => self.last_error = err.to_string(),
                }

                let fut = async move {
                    let res = socket.receive_message().await;
                    Msg::PollMessagesResult { socket, res }
                };
                ctx.link().send_future(fut);
                true
            }
            Msg::Nothing => false,
        }
    }

    fn view(&self, ctx: &Context<Model>) -> Html {
        let on_input_enter = ctx.link().callback(move |e: KeyboardEvent| {
            (e.key() == "Enter")
                .then(|| {
                    let input: InputElement = e.target_unchecked_into();
                    let value = input.value();
                    input.set_value("");
                    Msg::SendMessage(value)
                })
                .unwrap_or(Msg::Nothing)
        });

        html! {
            <div>
                { for self.messages.iter().map(|msg| html! { <p class="message">{msg}</p> }) }
                <input class="message_composer" type="text" value={self.current_message.clone()} onkeypress={on_input_enter}/>
                <p class="error">{format!("last error: {}", self.last_error.is_empty().then(|| "nothing").unwrap_or(&self.last_error))}</p>
            </div>
        }
    }
}

fn main() -> Result<(), BoxError> {
    let transport = Wasm::new("http://0.0.0.0:2289".parse().unwrap())?;

    let mut client = ChatClient::new_transport(transport);
    let connect_socket_fut = client
        .stream_messages(Empty {})
        .map(Msg::SocketCreateResult);

    let app = yew::start_app_with_props::<Model>(Props {
        client: RefCell::new(client),
    });
    app.send_future(connect_socket_fut);

    Ok(())
}
