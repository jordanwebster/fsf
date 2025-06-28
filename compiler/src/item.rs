use crate::expression::BlockExpression;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: String,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_annotation: String,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function {
        name: String,
        parameters: Vec<Parameter>,
        body: BlockExpression,
        return_type: Option<String>,
    },
    // TODO: Merge into function once we have typing
    Component {
        name: String,
        parameters: Vec<Parameter>,
        body: BlockExpression,
    },
    Import {
        path: Vec<String>,
    },
    TestRunner,
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
}
