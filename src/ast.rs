use crate::span::Span;

pub enum Vis {
    Priv,
    Pub(Span),
}

pub struct Id {
    pub span: Span,
    pub value: String,
}

pub enum PatternBody {
    Id(Id),
}

pub struct Pattern {
    pub span: Span,
    pub body: PatternBody,
}

pub enum TypeBody {
    Named(Id),
}

pub struct Type {
    pub span: Span,
    pub body: TypeBody,
}

pub struct Param {
    pub span: Span,
    pub name: Pattern,
    pub ty: Type,
}

pub struct Fn {
    pub vis: Vis,
    pub name: Id,
    pub params: Vec<Param>,
}

pub struct Alias {
    pub vis: Vis,
    pub new: Type,
    pub under: Type,
}

pub enum ItemBody {
    Alias(Alias),
    Fn(Fn),
}

pub struct Item {
    pub span: Span,
    pub body: ItemBody,
}

pub struct File {
    pub span: Span,
    pub items: Vec<Item>,
}
