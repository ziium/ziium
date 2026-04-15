#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Bind {
        name: String,
        value: Expr,
        mutable: bool,
    },
    Assign {
        name: String,
        value: Expr,
    },
    Print {
        value: Expr,
    },
    Sleep {
        duration_seconds: Expr,
    },
    Return {
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    FunctionDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    KeywordMessage {
        receiver: Expr,
        selector: String,
        arg: Expr,
    },
    Resultive {
        receiver: Expr,
        role: String,
        verb: String,
    },
    NamedCall {
        callee: Expr,
        named_args: Expr,
    },
    IndexAssign {
        base: String,
        index: Expr,
        value: Expr,
    },
    ForEach {
        collection: Expr,
        variable: String,
        body: Vec<Stmt>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Name(String),
    Int(String),
    Float(String),
    String(String),
    Bool(bool),
    None,
    List(Vec<Expr>),
    Record(Vec<RecordEntry>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        form: BinarySurface,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    TransformCall {
        input: Box<Expr>,
        callee: String,
    },
    Resultive {
        receiver: Box<Expr>,
        role: String,
        verb: String,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    Property {
        base: Box<Expr>,
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinarySurface {
    Symbol,
    Word,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEntry {
    pub key: String,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}
