use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::services::{ConsoleService};
use stdweb::web::event::KeyDownEvent;
use stdweb::web::event::IKeyboardEvent;
use stdweb::web::event::IEvent;

mod content;
use content::{Content, CursorPos, GetString};

pub struct Model {
    console: ConsoleService,
    link: ComponentLink<Model>,
    text: String,
    cursor: CursorPos,
    cycle: Vec<CursorPos>,
    cycle_id: usize,
    content: Content,
}

pub enum Msg {
    GotInput(String),
    KeyEvt(KeyDownEvent),
    Ignore,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let typed = "fn test(&self,  other:\n  \n&mut usize){let x=(self+1)*other;\n return1<y}";
        //let typed = "fn test(&self, other:&mut usize){let x=(self+1)*other;return1<y}";
        let visible = "fn test(&self, other: &mut usize) {\n    let x = (self + 1) * other;\n    return 1 < y\n}";
        let content = Content::from_strings(&typed, &visible);
        
        Model {
            console: ConsoleService::new(),
            link,
            text: content.get_string(), //"fn test() {\n    println!(\"hello\")\n}".to_string(),
            cursor: content.cursor_pos(), //CursorPos { line: 0, col: 9, between: true },
            cycle: vec!(
                CursorPos { line: 0, col: 9, between: true },
                CursorPos { line: 0, col: 9, between: false },
                CursorPos { line: 0, col: 8, between: false },
                CursorPos { line: 0, col: 0, between: false },
                CursorPos { line: 1, col: 0, between: false },
                CursorPos { line: 1, col: 4, between: false },
                CursorPos { line: 1, col: 6, between: false },
                CursorPos { line: 1, col: 21, between: false },
            ),
            cycle_id: 0,
            content
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::KeyEvt(e) => {
                e.stop_propagation();
                e.prevent_default();
                match e.key().as_ref() {
                    "ArrowLeft" => {
                        // self.cycle_id = (self.cycle_id + self.cycle.len() - 1) % self.cycle.len();
                        // self.cursor = self.cycle[self.cycle_id].clone();
                        self.content.cursor_left();
                        self.cursor = self.content.cursor_pos();
                    },
                    "ArrowRight" => {
                        // self.cycle_id = (self.cycle_id + 1) % self.cycle.len();
                        // self.cursor = self.cycle[self.cycle_id].clone();
                        self.content.cursor_right();
                        self.cursor = self.content.cursor_pos();
                    },
                    "Backspace" => {
                        self.content.backspace();
                        self.cursor = self.content.cursor_pos();
                        self.text = self.content.get_string();
                        //self.console.log(&format!("{:?}", self.view_model.to_model_pos(false)));
                        //let req = self.view_model.backspace();
                        //self.ws.as_mut().unwrap().send(Json(&req));
                    },
                    "Enter" => {
                        self.content.insert('\n');
                        self.cursor = self.content.cursor_pos();
                        self.text = self.content.get_string();
                    },
                    x if x.len() == 1 => {
                        self.content.insert(x.chars().next().unwrap());
                        self.cursor = self.content.cursor_pos();
                        self.text = self.content.get_string();
                        //self.console.log(&format!("{:?}", self.view_model.to_model_pos(true)));
                        //let req = self.view_model.insert(x.to_string());
                        //self.ws.as_mut().unwrap().send(Json(&req));
                    },
                    _ => ()
                }
                self.console.log(&format!("{:?}", e.key()));
                // FIXME: implement
                
            },
            Msg::GotInput(s) => {
                //self.input = s;
            },
            Msg::Ignore => {
                return false;
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let col = self.cursor.col as f32 + if self.cursor.between { 0.5 } else { 0.0 };
        let x = (col * 10.0);
        let y = self.cursor.line as f32 * 19.0;
        let s = format!("background-color: grey; position: absolute; width: 2px; height: 19px; top: {}px; left: {}px;", y, x-1.0);
        html! {
            <div  >
                /*<nav class="menu",>
                    <button disabled=self.ws.is_some(),
                            onclick=|_| WsAction::Connect.into(),>{ "Connect To WebSocket" }</button>
                    <button disabled=self.ws.is_none(),
                            onclick=|_| WsAction::Disconnect.into(),>{ "Close WebSocket connection" }</button>
                </nav>*/
                /*<textarea rows=5, style="width: 100%", 
                    oninput=|e| WsAction::SendData(e.value).into(),
                    placeholder="placeholder",>
                </textarea>
                <p></p>
                <textarea rows=10, placeholder="output goes here", readonly=true, style="width: 100%", >
                    { &self.output }
                </textarea>
                */
                <div style="width:80%; border: 1px solid black; padding: 10px;", onkeydown=|e| Msg::KeyEvt(e), tabindex="0", >
                    <div style="font-family: monospace; position: relative; font-size: 12pt;", >
                        <pre>{ self.text.clone() }</pre>
                        <div id="cursor", style=s, ></div>
                    </div>
                </div>
                <span style="font-family: monospace; position: relative; font-size: 12pt;", > {"x"} </span>
                //{ format!("{:?}", self.view_model.pos) }
            </div>
        }
    }
}