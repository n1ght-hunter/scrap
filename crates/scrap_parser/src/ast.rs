#[derive(Debug, Clone, Copy)]
pub struct Symbol(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new() -> Self {
        NodeId(0)
    }
}

// #[derive(Debug, Clone)]
// pub enum Literal {
//     Int(&str),
//     Float(&str),
//     Str(&str),
// }
