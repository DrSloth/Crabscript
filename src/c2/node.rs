use crate::{
    base::{Args, DayFunction, DayObject, RustFunction},
    manager::RuntimeManager,
    std_modules::{conversion::to_bool_inner, iter::to_iter_inner},
};
use std::sync::Arc;

//TODO Closures, The rest of the nodes, Consts

type NodeJump = unsafe fn(&Node, &Arc<RuntimeManager>) -> ExpressionResult;

//IMPORTANT The Order of NODE_JUMPS and all other jump tables is important.
//Check out all IMPORTANT annotations before changing anything

const NODE_JUMPS: [NodeJump; 14] = [
    //Node::RustFunction
    exec_rust_fn,
    //NODE::Identifier
    exec_ident,
    //Node::Data
    exec_data,
    //Node::FunctionCall
    exec_call,
    //Node::For
    exec_for,
    //Node::Assignment
    exec_assignment,
    //Node::Declaration
    exec_decl,
    //NOTE Currently Const declarations are only important in the parser
    //This could be changed in the future and has to be asserted
    //Node::ConstDeclaration
    exec_decl,
    //Node::BranchNode
    exec_branch,
    //Node::While
    exec_while,
    //Node::Block
    exec_block,
    //Node::Ret
    exec_ret,
    //Node::Index
    exec_index,
    //Node::Index
    exec_function_decl,
];

#[repr(u8)]
#[derive(Debug)]
pub enum Node {
    RustFunction(ConstRustFn),
    Identifier(IdentifierNode),
    Data(DayObject),
    FunctionCall(FunctionCallNode),
    For {
        expr: Box<Node>,
        block: Block,
    },
    Assignment {
        assignee: Box<Node>,
        value: Box<Node>,
    },
    Declaration {
        value: Box<Node>,
        id: usize
    },
    ConstDeclaration {
        value: Box<Node>,
        id: usize
    },
    BranchNode(Vec<BranchNode>),
    While {
        condition: Box<Node>,
        block: Block,
    },
    Block(Block),
    Ret(Option<Arc<Node>>),
    Index(IndexNode),
    FunctionDeclaration {
        block: Arc<Block>,
        is_closure: bool,
    },
}

impl Node {
    pub fn execute(&self, manager: &Arc<RuntimeManager>) -> ExpressionResult {
        let tag: u8 = unsafe { std::mem::transmute_copy(self) };
        unsafe { NODE_JUMPS[tag as usize](self, manager) }
    }

    pub fn function_decl(block: Block, is_closure: bool) -> Self {
        Self::FunctionDeclaration {
            block: Arc::new(block),
            is_closure,
        }
    }
}

unsafe fn exec_data(data: &Node, _manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@data");
    let data = data as *const _ as *const (u8, DayObject);
    ExpressionResult::Value((*data).1.clone())
}

unsafe fn exec_rust_fn(rust_fn: &Node, _manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@rfn");
    if let Node::RustFunction(rfn) = rust_fn {
        return ExpressionResult::Value(DayObject::Function(DayFunction::Function(rfn.0.clone())));
    }
    std::hint::unreachable_unchecked();
}

type CallJump = unsafe fn(&FunctionCallNode, &Arc<RuntimeManager>) -> ExpressionResult;

const CALL_JUMPS: [CallJump; 3] = [call_rustfn, call_ident, call_other];

unsafe fn get_args<'a>(
    call: &'a FunctionCallNode,
    manager: &Arc<RuntimeManager>,
) -> &'a [DayObject] {
    let cache = &mut *call.arg_cache.get();
    if cache.len() != call.args.len() {
        for a in &call.args {
            cache.push(a.execute(manager).value())
        }
    } else {
        for (a, n) in cache.iter_mut().zip(call.args.iter()) {
            *a = n.execute(manager).value()
        }
    }
    &*cache
}

unsafe fn call_rustfn(call: &FunctionCallNode, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@crfn");
    let (_, rfn) = &*(&*call.expr as *const _ as *const (u8, ConstRustFn));

    let args = get_args(call, manager);
    return ExpressionResult::Value(rfn.0(args));
}

unsafe fn call_ident(call: &FunctionCallNode, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@cid");
    if let Node::Identifier(id) = &*call.expr {
        match id.get_mut(manager) {
            DayObject::Function(func) => {
                return ExpressionResult::Value(func.call(get_args(call, manager)))
            }
            DayObject::Iter(handle) => {
                if let Some(obj) = handle.0.next() {
                    return ExpressionResult::Value(obj);
                } else {
                    return ExpressionResult::Value(DayObject::None);
                }
            }
            _ => panic!("Err: The function {:?} does not exist!", id),
        }
    }
    std::hint::unreachable_unchecked();
}

unsafe fn call_other(call: &FunctionCallNode, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@cother");
    match call.expr.execute(manager).value() {
        DayObject::Function(func) => {
            return ExpressionResult::Value(func.call(get_args(call, manager)))
        }
        DayObject::Iter(mut handle) => {
            if let Some(obj) = handle.0.next() {
                return ExpressionResult::Value(obj);
            } else {
                return ExpressionResult::Value(DayObject::None);
            }
        }
        other => panic!("Can't call {:?}", other),
    }
}

unsafe fn exec_call(call: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@call");
    let callptr = call as *const _ as *const (u8, FunctionCallNode);
    let jmp: u8 = std::mem::transmute_copy(&*(*callptr).1.expr);
    let jmp: usize = jmp as usize;
    dbg_print!(jmp);
    if jmp < 2 {
        CALL_JUMPS[jmp](&(*callptr).1, manager)
    } else {
        CALL_JUMPS[2](&(*callptr).1, manager)
    }
}

unsafe fn exec_for(for_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@for");
    if let Node::For { expr, block } = for_node {
        let mut iter = to_iter_inner(&expr.execute(manager).value());

        //TODO It has to be asserted that an ident is only used after definition
        //This could (and probably should) be done in the parser such that use before
        //definition is a preexecution parsing error

        while let Some(i) = iter.0.next() {
            block.scope.def_var(0, i);
            block.execute();
        }

        block.scope.clear();


        return ExpressionResult::Value(DayObject::None);
    }
    std::hint::unreachable_unchecked()
}

unsafe fn exec_assignment(
    assignment_node: &Node,
    manager: &Arc<RuntimeManager>,
) -> ExpressionResult {
    if let Node::Assignment { assignee, value: v } = assignment_node {
        match &**assignee {
            Node::Identifier(id) => {
                let value = v.execute(manager).value();
                id.set_var(manager, value);
                return ExpressionResult::Value(DayObject::None);
            }
            Node::Index(inner) => {
                *inner.get_mut(&manager) = v.execute(manager).value();
                return ExpressionResult::Value(DayObject::None);
            }
            other => panic!("Can't assign to {:?}", other),
        }
    }
    std::hint::unreachable_unchecked()
}

unsafe fn exec_decl(decl_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@decl");
    if let Node::Declaration { value: v, id } = decl_node {
        let value = v.execute(manager).value();
        manager.def_var(*id, value);
        return ExpressionResult::Value(DayObject::None);
    }
    std::hint::unreachable_unchecked()
}

unsafe fn exec_ident(ident_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    dbg_print_pretty!("@id");
    let (_, id) = &*(ident_node as *const _ as *const (u8, IdentifierNode));
    let val = id.get_var(manager);
    ExpressionResult::Value(val)
}

unsafe fn exec_branch(branch_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    let (_, branches) = &*(branch_node as *const _ as *const (u8, Vec<BranchNode>));
    for b in branches {
        if let BranchNode::Else { block } = b {
            return block.execute();
        } else {
            let (_, ifb) = &*(b as *const _ as *const (u8, IfBlock));
            if let Some(res) = ifb.execute(manager) {
                return res;
            }
        }
    }

    ExpressionResult::Value(DayObject::None)
}

unsafe fn exec_while(while_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    if let Node::While{condition, block} = while_node {
        while to_bool_inner(&condition.execute(manager).value()) {
            block.execute();
        }
        block.scope.clear()
    }

    return ExpressionResult::Value(DayObject::None);
}

unsafe fn exec_block(block_node: &Node, _manager: &Arc<RuntimeManager>) -> ExpressionResult {
    if let Node::Block(blk) = block_node {
        return blk.execute();
    }

    std::hint::unreachable_unchecked();
}

unsafe fn exec_ret(ret_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    if let Node::Ret(blk) = ret_node {
        if let Some(node) = blk { 
            return ExpressionResult::Return(node.execute(manager).value())
        } else {
            return ExpressionResult::Return(DayObject::None)
        }
    }

    std::hint::unreachable_unchecked();
}

unsafe fn exec_index(index_node: &Node, manager: &Arc<RuntimeManager>) -> ExpressionResult {
    if let Node::Index(ind) = index_node {
        return ExpressionResult::Value(ind.get_value(manager))
    }

    std::hint::unreachable_unchecked();
}

unsafe fn exec_function_decl(_fdecl_node: &Node, _manager: &Arc<RuntimeManager>) -> ExpressionResult {
    todo!()
    /* if let Node::FunctionDeclaration { block, is_closure: _} = fdecl_node {
    }

    std::hint::unreachable_unchecked(); */
}

//------------------------------------------------------------------
//------------------------------------------------------------------
//SECTION
//------------------------------------------------------------------
//------------------------------------------------------------------

#[repr(u8)]
#[derive(Debug)]
pub enum BranchNode {
    If { block: IfBlock },
    ElseIf { block: IfBlock },
    Else { block: Block },
}

#[derive(Debug)]
pub struct IfBlock {
    pub condition: Box<Node>,
    pub block: Block,
}

impl IfBlock {
    fn execute(&self, manager: &Arc<RuntimeManager>) -> Option<ExpressionResult> {
        if let DayObject::Bool(true) = self.condition.execute(manager).value() {
            Some(self.block.execute())
        } else {
            None
        }
    }
}

pub struct Block {
    pub block: RootNode,
    pub scope: Arc<RuntimeManager>,
}

impl Block {
    pub fn new(purpose: NodePurpose, scope: Arc<RuntimeManager>) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope,
        }
    }

    pub fn new_capacity(purpose: NodePurpose, inner_capacity: usize) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope: Arc::new(RuntimeManager::new_capacity(inner_capacity)),
        }
    }

    pub fn new_capacity_predecessor(
        purpose: NodePurpose,
        inner_capacity: usize,
        predecessor: Option<Arc<RuntimeManager>>,
    ) -> Self {
        Self {
            block: RootNode {
                purpose,
                nodes: Default::default(),
            },
            scope: Arc::new(RuntimeManager::new_capacity_predecessor(
                inner_capacity,
                predecessor,
            )),
        }
    }

    pub fn push(&mut self, node: Node) {
        self.block.push(node)
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.block.pop()
    }

    pub fn len(&self) -> usize {
        self.block.len()
    }

    pub fn execute(&self) -> ExpressionResult {
        self.block.execute(&self.scope)
    }

    pub fn execute_args(&self, args: Args) -> ExpressionResult {
        self.scope.def_args_alloc(args.to_vec());
        self.block.execute(&self.scope)
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        //FIXME Dummy impl
        write!(f, "{:#?}", self.block)
    }
}

#[derive(Debug)]
pub struct RootNode {
    nodes: Vec<Node>,
    pub purpose: NodePurpose,
}

impl<'a: 'v, 'v, 's> RootNode {
    pub fn new(purpose: NodePurpose) -> Self {
        Self {
            nodes: Default::default(),
            purpose,
        }
    }

    pub fn push(&mut self, node: Node) {
        self.nodes.push(node)
    }

    pub fn pop(&mut self) -> Option<Node> {
        self.nodes.pop()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn execute(&'s self, manager: &Arc<RuntimeManager>) -> ExpressionResult {
        for n in self.nodes.iter() {
            match n.execute(manager) {
                ExpressionResult::Return(res) => {
                    return if self.purpose == NodePurpose::Function {
                        ExpressionResult::Value(res)
                    } else {
                        ExpressionResult::Return(res)
                    }
                }
                v => v,
            };
        }

        ExpressionResult::Value(DayObject::None)
    }
}

//Prolly args should be an extra Node

use std::cell::UnsafeCell;

#[repr(C)]
#[derive(Debug)]
pub struct IdentifierNode {
    pub id: usize,
    pub depth: usize,
}

impl IdentifierNode {
    pub fn new(id: usize, depth: usize) -> Self {
        Self { id, depth }
    }

    pub fn get_var(&self, manager: &Arc<RuntimeManager>) -> DayObject {
        manager.get_var(self.id, self.depth)
    }

    pub fn get_mut(&self, manager: &Arc<RuntimeManager>) -> &mut DayObject {
        manager.get_var_mut(self.id, self.depth)
    }

    pub fn set_var(&self, manager: &Arc<RuntimeManager>, value: DayObject) {
        manager.set_var(value, self.id, self.depth)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodePurpose {
    Function,
    Conditional,
    While,
    Block,
    TopLevel,
    For,
}

impl<'a> IntoIterator for RootNode {
    type Item = Node;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

#[derive(Debug)]
pub struct IndexOperation {
    pub index: Box<Node>,
}

#[derive(Debug, Clone)]
pub enum ExpressionResult {
    Return(DayObject),
    Value(DayObject),
    Yielded(DayObject),
}

impl ExpressionResult {
    ///Retrievs the value if this is the value variant
    pub fn value(self) -> DayObject {
        match self {
            Self::Value(d) => d,
            er => panic!("Expected value received {:?}", er),
        }
    }
}

#[derive(Debug)]
pub struct IndexNode {
    pub initial: Box<Node>,
    pub index_ops: Vec<IndexOperation>,
}

impl IndexNode {
    pub fn get_value(&self, manager: &Arc<RuntimeManager>) -> DayObject {
        self.get_mut(manager).clone()
    }

    pub fn get_mut(&self, manager: &Arc<RuntimeManager>) -> &mut DayObject {
        match &*self.initial {
            Node::Identifier(IdentifierNode { id, depth }) => {
                manager.get_var_mut(*id, *depth)
            },
            _ => todo!()
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct FunctionCallNode {
    pub expr: Box<Node>,
    pub args: Vec<Node>,
    pub arg_cache: UnsafeCell<Vec<DayObject>>,
}

use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone)]
pub struct ConstRustFn(pub RustFunction);

impl Debug for ConstRustFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "RustFunction")
    }
}
