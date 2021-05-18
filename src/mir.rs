use std::{cell::RefCell, collections::HashMap, env::var, ops::Deref, rc::Rc};

use crate::nbf::{self, codegen::{CodeGen, StaticVar}};

pub struct Id(pub usize);

pub enum Value {
    Const(u32),
    StaticVar(Id)
}

pub enum Statement {
    With(Box<Statement>),
    Seq(Vec<Statement>),
    AddAssign(Id, Value),
    Set(Id, Value),
    WhileNeq0(Id, Box<Statement>),
    Print(Id),
    Read(Id)
}

impl Statement {

    pub fn to_nbf(&self) -> nbf::Command {
        let mut frame = Frame::new();
        let mut codegen = CodeGen::new(100);
        self.to_nbf_impl(frame, &mut codegen);
        codegen.compile_nbf()
    }

    fn to_nbf_impl(&self, mut frame: Frame, codegen: &mut CodeGen) -> Frame {
        use nbf::codegen::CodeGenerator;

        match self {
            Statement::With(body) => {
                let mut var = codegen.static_alloc().unwrap();
                codegen.zero(&mut var);
                let frame = Frame::declare(frame, var);
                let frame = body.to_nbf_impl(frame, codegen);
                if let Frame::Node { previous, var } = frame {
                    codegen.static_free(var);
                    *previous
                } else { unreachable!() }
            }
            Statement::Seq(statements) => {
                for statement in statements {
                    frame = statement.to_nbf_impl(frame, codegen);
                }
                frame
            }
            Statement::AddAssign(_, _) => { frame }
            Statement::Set(_, _) => { frame }
            Statement::WhileNeq0(id, body) => {
                let mut var = frame.fetch(id.0).unwrap();
                codegen.while_neq0(var, |codegen, var| {
                    
                })
            }
            Statement::Print(id) => {
                let var = frame.fetch(id.0).unwrap();
                codegen.print(var);
                frame
            }
            Statement::Read(id) => {
                let var = frame.fetch(id.0).unwrap();
                codegen.read(var);
                frame
            }
        }
    }
}

enum Frame {
    Empty,
    Node {
        var: Rc<RefCell<StaticVar>>,
        previous: Box<Frame>
    }
}

impl Frame {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn declare(self, var: StaticVar) -> Self {
        Self::Node { var: Rc::new(RefCell::new(var)), previous: Box::new(self) }
    }

    pub fn fetch(&mut self, level: usize) -> Option<Rc<RefCell<StaticVar>> {
        match self {
            Self::Empty => None,
            Self::Node{var, ..} if level == 0 => Some(var),
            Self::Node{previous, ..} => previous.fetch(level - 1) 
        }
    }

    /// Panics if level_bigger <= level_smaller
    fn fetch_two(&mut self, level_bigger: usize, level_smaller: usize) -> Option<(&mut StaticVar, &mut StaticVar)> {
        assert!(level_bigger <= level_smaller);

        match self {
            Self::Empty => None,
            Self::Node {var, previous}  => {
                if level_smaller == 0 { 
                    previous
                        .fetch(level_bigger - 1)
                        .map(|bigger_var| (bigger_var, var))
                 } else {
                    previous.fetch_two(level_bigger - 1, level_smaller - 1)
                 }
            }
        }
    }
}