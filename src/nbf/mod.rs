pub mod codegen;


pub struct ExecutionContext {
    tape: Vec<u32>,
    position: usize,
    output: Vec<u32>,
    input: Vec<u32>,
    input_position: usize
}

impl ExecutionContext {
    pub fn new(input: &[u32]) -> Self {
        Self {
            tape: vec![0],
            position: 0,
            output: Vec::new(),
            input: input.to_owned(),
            input_position: 0
        }
    }

    pub fn move_left(&mut self) {
        if self.position != 0 {
            self.position -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.position + 1 >= self.tape.len() {
            self.tape.push(0);
        }
        self.position += 1;
    }

    pub fn move_i(&mut self, i: i32) {
        if i > 0 {
            for _ in 0..(i as usize) { self.move_right() }
        } else if i < 0 {
            for _ in 0..(i.abs() as usize) { self.move_left() }
        }
    }

    pub fn add(&mut self, i: i32) {
        self.tape[self.position] = if i > 0 {
            self.tape[self.position].wrapping_add(i.abs() as u32)
        } else {
            self.tape[self.position].wrapping_sub(i.abs() as u32)
        }
    }

    pub fn get_tape(&self) -> &u32 {
        &self.tape[self.position]
    }

    pub fn get_input(&mut self) -> u32 {
        if self.input_position >= self.input.len() { 0 } 
        else {
            let input = self.input[self.input_position];
            self.input_position += 1;
            input
        } 
    }

    pub fn read(&mut self) {
        let input = self.get_input();
        self.tape[self.position] = input;
    }


}

#[derive(Clone)]
pub enum Command {
    Add(i32),
    Move(i32),
    Loop(Box<Command>),
    Seq(Vec<Command>),
    Debug(DebugCmd),
    Print,
    Read
}

#[derive(Clone)]
pub enum DebugCmd {
    AssertPosition(usize)
}

impl DebugCmd {
    fn run(&self, context: &ExecutionContext) {
        match self {
            Self::AssertPosition(pos) => assert_eq!(*pos, context.position)
        }
    }
}

impl Command {
    pub fn to_repr(&self) -> String {
        todo!()
    }

    pub fn to_brainfuck(&self) -> String {
        todo!()
    }

    pub fn run(&self, input: &[u32]) -> Vec<u32>{
        let mut context = ExecutionContext::new(input);
        self.run_with_context(&mut context);
        context.output
    }

    fn run_with_context(&self, context: &mut ExecutionContext) {
        match self {
            Command::Add(val) => context.add(*val), 
            Command::Move(i) => context.move_i(*i),
            Command::Loop(body) => {
                while *context.get_tape() != 0 {
                    body.run_with_context(context)
                }
            }
            Command::Seq(commands) => {
                for command in commands {
                    command.run_with_context(context)
                }
            }
            Command::Debug(cmd) => cmd.run(context), 
            Command::Print => context.output.push(*context.get_tape()),
            Command::Read =>  context.read()
        }
    }
}