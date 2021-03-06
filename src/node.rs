use crate::{
    Accessor, Data, GlobalBorrow, Ident, InvokeRef, MacroInvokeRef, Type, TypeNode, ValueRef,
    INVOKES, VALUES,
};

#[derive(Debug, Clone)]
pub(crate) enum ValueNode {
    Tuple(Vec<ValueRef>),
    Str(String),
    // TODO: Add lifetime parameter
    Reference {
        is_mut: bool,
        value: ValueRef,
    },
    Dereference(ValueRef),
    Binding {
        name: Ident,
        ty: Type,
    },
    DataStructure {
        name: String,
        data: Data<ValueRef>,
    },
    Invoke(InvokeRef),
    Destructure {
        parent: ValueRef,
        accessor: Accessor,
        ty: Type,
    },
    MacroInvocation(MacroInvokeRef),
}

impl ValueNode {
    pub fn get_type(&self) -> Type {
        match self {
            ValueNode::Tuple(types) => Type(TypeNode::Tuple(
                types.iter().map(|type_ref| type_ref.get_type().0).collect(),
            )),
            ValueNode::Str(_) => Type(TypeNode::PrimitiveStr),
            ValueNode::Reference { is_mut, value } => Type(TypeNode::Reference {
                is_mut: *is_mut,
                lifetime: None,
                inner: Box::new(value.get_type().0),
            }),
            ValueNode::Binding { ty, .. } => ty.clone(),
            ValueNode::Destructure {
                parent,
                accessor,
                ty,
            } => ty.clone(),
            ValueNode::Invoke(invoke_ref) => {
                INVOKES.with_borrow(|invokes| invokes[invoke_ref.0].function.sig.output.clone())
            }

            node => panic!("ValueNode::get_type"),
        }
    }

    // FIXME: Consider generating invocations to std::any::type_name(), and
    // resolving generic parameters during the type and trait inference stage.
    pub fn get_type_name(&self) -> Self {
        match self {
            ValueNode::Tuple(types) => {
                let types: String =
                    types
                        .iter()
                        .fold(String::from(""), |mut acc, v| match &v.get_type_name() {
                            ValueNode::Str(name) => {
                                acc.push_str(name);
                                acc.push_str(", ");
                                acc
                            }
                            _ => unreachable!(),
                        });
                let types = format!("({})", types.trim_end_matches(", "));
                ValueNode::Str(types)
            }
            ValueNode::Str(_) => ValueNode::Str(String::from("str")),
            ValueNode::DataStructure { name, .. } => ValueNode::Str(name.to_owned()),
            ValueNode::Reference { value, .. } => value.get_type_name(),
            ValueNode::Binding { ty, .. } => ValueNode::Str(ty.0.get_name()),
            ValueNode::Destructure {
                parent,
                accessor,
                ty,
            } => ValueNode::Str(ty.0.get_name()),
            ValueNode::Invoke(invoke_ref) => ValueNode::Str(
                INVOKES
                    .with_borrow(|invokes| invokes[invoke_ref.0].function.sig.output.0.get_name()),
            ),
            node => panic!("ValueNode::get_type_name"),
        }
    }
}

impl ValueRef {
    pub(crate) fn get_type(self) -> Type {
        VALUES.with_borrow(|values| values[self.0].get_type())
    }

    pub(crate) fn get_type_name(self) -> ValueNode {
        VALUES.with_borrow(|values| values[self.0].get_type_name())
    }
}
