
use std::string::ToString;

#[derive(Clone, Debug)]
pub struct Content {
    elmts: Vec<Elmt>,
    cursor: (usize, usize),  // first element is the index of the selected whitespace element.
                             // the sectond element is the selection index within that whitespace element
}

#[derive(Clone, Debug)]
pub struct Elmt {
    character: char,
    whitespace: Whitespace,  // whitespace that's preceeding the character
}

#[derive(Clone, Debug)]
pub struct Whitespace {
    typed: Vec<WhitespaceChar>,
    virtual_newlines: usize,
    virtual_spaces: usize,  // on last line
}

#[derive(Clone, Debug)]
pub enum WhitespaceChar {
    Space,
    Newline
}

type CursorPos = ((usize, usize), (usize, usize));


impl WhitespaceChar {
    fn is_newline(&self) -> bool {
        match self {
            WhitespaceChar::Newline => true,
            _ => false,
        }
    }
}


pub trait GetString {
    fn get_string(&self) -> String;   // TODO: add option for visible whitespace
}

impl GetString for WhitespaceChar {
    fn get_string(&self) -> String {
        // visible whitespace characters · ¶
        match self {
            WhitespaceChar::Space => " ".to_string(),
            WhitespaceChar::Newline => "\n".to_string(),
        }
    }
}

impl GetString for Whitespace {
    fn get_string(&self) -> String {
        let num_typed_newlines = self.typed.iter().filter(|x| x.is_newline()).count();
        let num_spaces_last_line = self.typed.iter().rev().take_while(|x| !x.is_newline()).count();
        let mut s: String = self.typed.iter().map(|x| x.get_string()).collect();
        
        if num_typed_newlines < self.virtual_newlines {
            s.push_str(&"\n".repeat(self.virtual_newlines - num_typed_newlines));
            s.push_str(&" ".repeat(self.virtual_spaces));
        } else if num_typed_newlines == self.virtual_newlines && num_spaces_last_line < self.virtual_spaces {
            s.push_str(&" ".repeat(self.virtual_spaces - num_spaces_last_line));
        }
        
        s
    }
}

impl GetString for Elmt {
    fn get_string(&self) -> String {
        let mut s = self.whitespace.get_string();
        if self.character != '\0' {
            s.push(self.character);
        }
        s
    }
}

impl GetString for Content {
    fn get_string(&self) -> String {
        self.elmts.iter().map(|x| x.get_string()).collect()
    }
}



impl Content {
    pub fn from_string(input: &str) -> Content {
        let mut chars = input.chars();
        let mut elmts = vec!();
        let mut current_whitespace = vec!();

        while let Some(c) = chars.next() {
            match c {
                ' ' => current_whitespace.push(WhitespaceChar::Space),
                '\n' => current_whitespace.push(WhitespaceChar::Newline),
                other => {
                    elmts.push(Elmt {
                        character: other,
                        whitespace: Whitespace {
                            typed: current_whitespace,
                            virtual_newlines: 0,
                            virtual_spaces: 0,
                        }
                    });
                    current_whitespace = vec!();
                }
            }
        }

        elmts.push(Elmt {
            character: '\0',
            whitespace: Whitespace {
                typed: current_whitespace,
                virtual_newlines: 0,
                virtual_spaces: 0,
            }
        });

        Content {
            elmts: elmts,
            cursor: (0, 0)
        }
    }

    pub fn update_virtual_whitespace_2(&mut self, formatted_input: &str) {
        let mut chars = formatted_input.chars().peekable();
        
        for elmt in &mut self.elmts {
            let mut virtual_newlines = 0;
            let mut virtual_spaces = 0;
            while let Some(vc) = chars.peek() {
                match vc {
                    '\n' => { virtual_newlines += 1; virtual_spaces = 0; chars.next(); },
                    ' ' => { virtual_spaces += 1; chars.next(); },
                    x if x == &elmt.character => { chars.next(); break; },
                    ',' => { println!("Ignoring comma"); chars.next(); },
                    _ => {
                        break;
                    }
                }
            }
            
            elmt.whitespace.virtual_newlines = virtual_newlines;
            elmt.whitespace.virtual_spaces = virtual_spaces;
        }
    }

    pub fn from_strings(typed: &str, visible: &str) -> Content {
        let mut content = Content::from_string(typed);
        content.update_virtual_whitespace_2(visible);
        content
    }

    pub fn cursor_pos(&self) -> CursorPos {
        let s: String = self.elmts.iter().take(self.cursor.0).map(|x| x.get_string()).collect();
        let mut line = s.chars().filter(|x| x == &'\n').count();
        let mut col = s.chars().rev().take_while(|x| x != &'\n').count();
        let mut between = false;
        
        let virtual_spaces = self.elmts[self.cursor.0].whitespace.virtual_spaces;
        let virtual_newlines = self.elmts[self.cursor.0].whitespace.virtual_newlines;
        let virtual_end = (line + virtual_newlines, if virtual_newlines==0 {col} else {0} + virtual_spaces);

        let typed = &self.elmts[self.cursor.0].whitespace.typed;
        for wc in typed.iter().take(self.cursor.1) {
            match wc {
                WhitespaceChar::Space => col += 1,
                WhitespaceChar::Newline => {col = 0; line += 1;},
            }
        }

        let start = (line, col);
        let end = if self.cursor.1 == self.elmts[self.cursor.0].whitespace.get_num_cursor_positions() - 1 {
            std::cmp::max(start, virtual_end)
        } else {
            start.clone()
        };
        
        (start, end)
    }

    pub fn cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor = (self.cursor.0 - 1, self.elmts[self.cursor.0 - 1].whitespace.get_num_cursor_positions() - 1);
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor.1 < self.elmts[self.cursor.0].whitespace.get_num_cursor_positions() - 1 {
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.elmts.len() - 1 {
            self.cursor = (self.cursor.0 + 1, 0);
        }
    }

    pub fn insert(&mut self, c: char) {
        // check for whitespace
        if c == '\n' || c == ' ' {
            let typed_len = self.elmts[self.cursor.0].whitespace.typed.len();
            let ws_char = if c == '\n' { WhitespaceChar::Newline } else { WhitespaceChar::Space };
            self.elmts[self.cursor.0].whitespace.typed.insert(std::cmp::min(self.cursor.1, typed_len), ws_char);
            self.cursor.1 += 1;
            return;
        }

        let mut ws_left = self.elmts[self.cursor.0].whitespace.typed.clone();
        let ws_left_len = ws_left.len();
        let ws_right = ws_left.split_off(std::cmp::min(self.cursor.1, ws_left_len));
        let new_elmt = Elmt {
            character: c,
            whitespace: Whitespace {
                typed: ws_left,
                virtual_newlines: 0,
                virtual_spaces: 0,
            }
        };
        self.elmts[self.cursor.0].whitespace.typed = ws_right;
        self.elmts.insert(self.cursor.0, new_elmt);
        self.cursor = (self.cursor.0 + 1, 0);
    }

    pub fn backspace(&mut self) {
        if self.cursor.1 > 0 {
            if self.cursor.1 <= self.elmts[self.cursor.0].whitespace.typed.len() {
                self.elmts[self.cursor.0].whitespace.typed.remove(self.cursor.1 - 1);
            }
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            let ws_left = &self.elmts[self.cursor.0 - 1].whitespace;
            let ws_right = &self.elmts[self.cursor.0].whitespace;
            let mut typed_new = ws_left.typed.clone();
            typed_new.extend(ws_right.typed.clone());
            let ws_new = Whitespace {
                typed: typed_new,
                virtual_newlines: ws_left.virtual_newlines + ws_right.virtual_newlines,
                virtual_spaces: if ws_right.virtual_newlines == 0 { 
                    ws_left.virtual_spaces + ws_right.virtual_spaces 
                } else {
                    ws_right.virtual_spaces
                },
            };
            self.elmts[self.cursor.0].whitespace = ws_new;
            let cursor_new = (self.cursor.0 - 1, self.elmts[self.cursor.0 - 1].whitespace.get_num_cursor_positions() - 1);
            self.elmts.remove(self.cursor.0 - 1);
            self.cursor = cursor_new;
        }
    }

    pub fn delete(&mut self) {
        unimplemented!();
    }

    pub fn clear_virtual_whitespace(&mut self) {
        for e in &mut self.elmts {
            e.whitespace.virtual_newlines = 0;
            e.whitespace.virtual_spaces = 0;
        }
    }

    pub fn update_virtual_whitespace(&mut self, window_width: usize) -> String {
        // get string (without virtual whitespace)
        let mut clone = self.clone();
        clone.clear_virtual_whitespace();
        let s = clone.get_string();

        // pass that string to rustfmt
        match prettify_code(s.clone(), window_width) {
            Some(res) => {
                self.update_virtual_whitespace_2(&res);
                self.cursor.1 = std::cmp::min(
                    self.cursor.1, 
                    self.elmts[self.cursor.0].whitespace.get_num_cursor_positions() - 1
                );
                res
            },
            None => "error".to_string()
        }
    }
}

impl Whitespace {
    fn get_num_cursor_positions(&self) -> usize {
        let num_typed_newlines = self.typed.iter().filter(|x| x.is_newline()).count();
        self.typed.len() + 1
    }
}

pub fn prettify_code(input: String, window_width: usize) -> Option<String> {
    let mut buf = Vec::new();
    {
        let mut config = rustfmt_nightly::Config::default();
        config.set().emit_mode(rustfmt_nightly::EmitMode::Stdout);
        config.set().edition(rustfmt_nightly::Edition::Edition2018);
        config.set().max_width(window_width);
        let mut session = rustfmt_nightly::Session::new(config, Some(&mut buf));
        session.format(rustfmt_nightly::Input::Text(input)).unwrap();
        if !session.has_no_errors() {
            return None
        }
    }
    Some(String::from_utf8(buf[8..].to_vec()).unwrap())
}

pub fn prettify_text(input: String, window_width:usize) -> Option<String> {
    let mut s = String::new();
    let mut width = 0;
    for word in input.split_whitespace(){
        if width + word.len() <= window_width {
            width += word.len() + 1;
            s.push_str(word);
            s.push(' ');
        } else {
            width = word.len() + 1;
            s.push('\n');
            s.push_str(word);
            s.push(' ');
        }
    }
    if !s.is_empty() {
        s.pop();
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let typed = "fn test(&self,other:&mut usize){let x=(self+1)*other;return1<y}";
        let visible = "fn test(&self, other: &mut usize) {\n    let x = (self + 1) * other;\n    return 1 < y\n}";
        let c = Content::from_strings(&typed, &visible);
        let s = c.get_string();
        assert_eq!(&s, visible);
    }

    #[test]
    fn test_extra_whitespace() {
        let typed = "fn test(&self,  other:\n  \n&mut usize){let x=(self+1)*other;\n return1<y}";
        let visible = "fn test(&self, other: &mut usize) {\n    let x = (self + 1) * other;\n    return 1 < y\n}";
        let out = "fn test(&self,  other:\n  \n&mut usize) {\n    let x = (self + 1) * other;\n    return 1 < y\n}";
        let c = Content::from_strings(&typed, &visible);
        let s = c.get_string();
        assert_eq!(&s, out);
    }

    #[test]
    fn test_visible_contains_extra_comma() {
        let typed = "let x = [a,b,c]";
        let visible = "let x = [a, b, c, ]";
        let out = "let x = [a, b, c ]";
        let c = Content::from_strings(&typed, &visible);
        let s = c.get_string();
        assert_eq!(&s, out);
    }
    
    #[test]
    fn test_visible_doesnt_contain_comma() {
        let typed = "let x = [a,b,c,];let y = 15;";
        let visible = "let x = [a, b, c];let y = 15;";
        let out = "let x = [a, b, c,];let y = 15;";
        let c = Content::from_strings(&typed, &visible);
        let s = c.get_string();
        assert_eq!(&s, out);
    }

    #[test]
    fn test_num_cursor_positions() {
        let ws = Whitespace {
            typed: vec!(),
            virtual_newlines: 0,
            virtual_spaces: 0,
        };
        assert_eq!(ws.get_num_cursor_positions(), 1);
    }
    #[test]
    fn test_num_cursor_positions_typed_only() {
        use WhitespaceChar::*;
        let ws = Whitespace {
            typed: vec!(Space, Space),
            virtual_newlines: 0,
            virtual_spaces: 0,
        };
        assert_eq!(ws.get_num_cursor_positions(), 3);
        
        let ws = Whitespace {
            typed: vec!(Newline),
            virtual_newlines: 0,
            virtual_spaces: 0,
        };
        assert_eq!(ws.get_num_cursor_positions(), 2);
    }

    #[test]
    fn test_num_cursor_positions_virtual_only() {
        let ws = Whitespace {
            typed: vec!(),
            virtual_newlines: 0,
            virtual_spaces: 3,
        };
        assert_eq!(ws.get_num_cursor_positions(), 1);
        
        let ws = Whitespace {
            typed: vec!(),
            virtual_newlines: 2,
            virtual_spaces: 10,
        };
        assert_eq!(ws.get_num_cursor_positions(), 3);
    }

    #[test]
    fn test_num_cursor_positions_mixed() {
        use WhitespaceChar::*;
        let ws = Whitespace {
            typed: vec!(Space, Space),
            virtual_newlines: 0,
            virtual_spaces: 5,
        };
        assert_eq!(ws.get_num_cursor_positions(), 3);
        
        let ws = Whitespace {
            typed: vec!(Space, Newline, Space),
            virtual_newlines: 2,
            virtual_spaces: 0,
        };
        assert_eq!(ws.get_num_cursor_positions(), 5);
    }

    #[test]
    fn test_backspace() {
        let mut content = Content::from_string("a \n  def");
        content.cursor_right();
        content.cursor_right();
        content.backspace();
        assert_eq!(&content.get_string(), "a\n  def");
    }

    #[test]
    fn test_delete_typed_whitespace() {
        let mut content = Content::from_string("a\n\n def");
        content.cursor_right();
        content.delete();
        assert_eq!(&content.get_string(), "a\n def");
        content.delete();
        assert_eq!(&content.get_string(), "a def");
        content.delete();
        assert_eq!(&content.get_string(), "adef");
    }

    #[test]
    fn test_delete_character(){
        let mut content = Content::from_string(" adef");
        content.cursor_right();
        content.delete();
        assert_eq!(&content.get_string(), " def");
        content.cursor_right();
        content.delete();
        assert_eq!(&content.get_string(), " df");
    }

    #[test]
    fn test_delete_virtual_whitespace(){
        let mut content = Content::from_string("a5");
        content.update_virtual_whitespace_2("a\n  5");
        content.cursor_right();
        content.delete();  // moves to next line
        assert_eq!(&content.get_string(), "a\n  5");
        content.delete();  // deletes "5"
        assert_eq!(&content.get_string(), "a\n  ");
    }

    
}
