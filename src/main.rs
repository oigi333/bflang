
mod nbf;
mod mir;

use nbf::codegen::{CodeGen, CodeGenerator};

fn main() {

    // let mut codegen = CodeGen::new(100);
    // let mut n = codegen.static_alloc().unwrap();
    // let mut a = codegen.static_alloc().unwrap();
    // let mut b = codegen.static_alloc().unwrap();
    // let mut temp = codegen.static_alloc().unwrap();

    // codegen.read(&n);
    // codegen.add_const(&mut a, 1);
    // codegen.while_neq0(&mut n, |codegen, n| {
    //     codegen.set(&mut temp, &mut a)?;
    //     codegen.add(&mut a, &mut b)?;
    //     codegen.set(&mut b, &mut temp)?;
    //     codegen.dec(n);
    //     Ok(())
    // }).unwrap();
    // codegen.print(&b);
        use mir::*;
    let statement = Statement::With(Box::new(Statement::Seq(vec![
        Statement::Read(Id(0)),
        Statement::With(Box::new(Statement::Seq(vec![
            Statement::Read(Id(0)),
            Statement::Read(Id(1)),
            Statement::Print(Id(0))
        ])))
    ])));

    let program = statement.to_nbf();
    let (mut a, mut b) = (1, 0);
    // let program = codegen.compile_nbf();
    //println!("{}", program.to_repr(true));
    println!("{}", program.to_brainfuck());
    for i in 0..60 {
        println!("i: {} = {:?}", b, program.run(&[i, i]));
        let temp = a;
        a = a + b;
        b = temp;
        
    }
}
