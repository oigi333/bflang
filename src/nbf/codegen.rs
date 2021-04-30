use std::{
    collections::BinaryHeap,
    ops::{Deref, DerefMut},
};

use crate::nbf::{Command, DebugCmd};

#[derive(Debug)]
pub enum CodeGenError {
    Oom,
}
pub type Result<T> = std::result::Result<T, CodeGenError>;

pub struct CodeGen {
    code: Vec<Command>,
    static_memory: StaticMemory,
    position: usize,
}

pub struct CodeGenScope<'a> {
    code: Vec<Command>,
    static_memory: &'a mut StaticMemory,
    position: &'a mut usize,
}

impl CodeGen {
    pub fn new(static_memory_length: usize) -> Self {
        Self {
            code: vec![],
            static_memory: StaticMemory::new(static_memory_length),
            position: 0,
        }
    }

    pub fn compile_nbf(self) -> Command {
        Command::Seq(self.code)
    }

    pub fn compile_brainfuck(self) -> String {
        self.compile_nbf().to_brainfuck()
    }
}

pub trait CodeGenerator {
    fn static_memory(&mut self) -> &mut StaticMemory;
    fn code(&mut self) -> &mut Vec<Command>;
    fn position(&mut self) -> &mut usize;
    fn new_scope(&mut self) -> CodeGenScope;

    // Variables

    fn static_alloc(&mut self) -> Result<StaticVar> {
        self.static_memory().alloc()
    }

    fn static_free(&mut self, var: StaticVar) {
        self.static_memory().free(var);
    }

    // Code generation

    fn goto(&mut self, var: &StaticVar) {
        let position = self.position();
        let diff = (var.0 as isize) - (*position as isize);

        if diff != 0 {
            *position = var.0;
            self.emit(Command::Move(diff as i32));
        }

        self.assert_position(var.0);
    }

    fn zero(&mut self, var: &mut StaticVar) {
        self.goto(var);
        self.emit(Command::Loop(Box::new(Command::Add(-1))));
        self.assert_position(var.0);
    }

    fn read(&mut self, var: &StaticVar) {
        self.goto(var);
        self.emit(Command::Read)
    }

    fn print(&mut self, var: &StaticVar) {
        self.goto(var);
        self.emit(Command::Print)
    }

    fn inc(&mut self, var: &mut StaticVar) {
        self.add_const(var, 1)
    }

    fn dec(&mut self, var: &mut StaticVar) {
        self.add_const(var, -1)
    }

    fn add_const(&mut self, var: &mut StaticVar, val: i32) {
        self.goto(var);
        self.emit(Command::Add(val));
    }

    fn add(&mut self, lhs: &mut StaticVar, rhs: &mut StaticVar) -> Result<()> {
        let mut temp = self.static_alloc()?;

        self.zero(&mut temp);
        self.while_neq0(rhs, |codegen, rhs| {
            codegen.inc(&mut temp);
            codegen.inc(lhs);
            codegen.dec(rhs);
            Ok(())
        })?;

        self.while_neq0(&mut temp, |codegen, temp| {
            codegen.inc(rhs);
            codegen.dec(temp);
            Ok(())
        })?;

        self.static_free(temp);
        Ok(())
    }

    fn set(&mut self, lhs: &mut StaticVar, rhs: &mut StaticVar) -> Result<()> {
        self.zero(lhs);
        self.add(lhs, rhs)
    }

    fn loop_unchecked<F: FnOnce(&mut CodeGenScope) -> Result<()>>(
        &mut self,
        body: F,
    ) -> Result<()> {
        let mut scope = self.new_scope();
        body(&mut scope)?;
        let command = Command::Loop(Box::new(Command::Seq(scope.code)));
        self.emit(command);
        Ok(())
    }

    fn while_neq0<F: FnOnce(&mut CodeGenScope, &mut StaticVar) -> Result<()>>(
        &mut self,
        var: &mut StaticVar,
        body: F,
    ) -> Result<()> {
        self.goto(var);
        self.loop_unchecked(|codegen| {
            codegen.assert_position(var.0);
            body(codegen, var)?;
            codegen.goto(var);
            Ok(())
        })?;
        self.assert_position(var.0);
        Ok(())
    }

    fn emit(&mut self, command: Command) {
        self.code().push(command)
    }

    fn assert_position(&mut self, position: usize) {
        self.emit(Command::Debug(DebugCmd::AssertPosition(position)))
    }
}

impl CodeGenerator for CodeGen {
    fn static_memory(&mut self) -> &mut StaticMemory {
        &mut self.static_memory
    }

    fn code(&mut self) -> &mut Vec<Command> {
        &mut self.code
    }

    fn position(&mut self) -> &mut usize {
        &mut self.position
    }

    fn new_scope(&mut self) -> CodeGenScope {
        CodeGenScope {
            code: vec![],
            position: &mut self.position,
            static_memory: &mut self.static_memory,
        }
    }
}

impl<'a> CodeGenerator for CodeGenScope<'a> {
    fn static_memory(&mut self) -> &mut StaticMemory {
        self.static_memory
    }

    fn code(&mut self) -> &mut Vec<Command> {
        &mut self.code
    }

    fn position(&mut self) -> &mut usize {
        self.position
    }

    fn new_scope(&mut self) -> CodeGenScope {
        CodeGenScope {
            code: vec![],
            position: &mut self.position,
            static_memory: &mut self.static_memory,
        }
    }
}

use std::cmp::Reverse;

pub struct StaticMemory {
    free: BinaryHeap<Reverse<usize>>,
}

#[derive(Debug)]
pub struct StaticVar(usize);

impl StaticMemory {
    fn new(length: usize) -> Self {
        Self {
            free: (0..length).map(Reverse).collect(),
        }
    }

    fn alloc_raw(&mut self) -> Result<usize> {
        self.free.pop().map(|ptr| ptr.0).ok_or(CodeGenError::Oom)
    }

    pub fn alloc(&mut self) -> Result<StaticVar> {
        self.alloc_raw().map(StaticVar)
    }

    fn free_raw(&mut self, ptr: usize) {
        self.free.push(Reverse(ptr))
    }

    pub fn free(&mut self, variable: StaticVar) {
        self.free_raw(variable.0);
    }
}
