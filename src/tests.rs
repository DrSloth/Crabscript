use super::run;

#[test]
fn arithmetics() {
    run("let x = add(1, 10, 4, sub(500, 200, 90))
    println(x)")
}

#[test]
fn arrays() {
    run("let a = array(20, 50, 210)
    println(a[0])
    let x = a[1]
    println(x)")
}

#[test]
fn consts() {
    //NOTE const X = readln() blocks forever maybe it would be cool to
    //give the in and out streams to the run function maybe a simple run function
    //and one where argv, instream and outstream can be specified
    run(r#"println("Enter your name:")
    const X = "Name"
    print("Your name is: ", X, "Goodbye", "\n")"#)
}

#[test]
#[should_panic]
fn const_test() {
    run("const X = 100
    println(X)
    X = 2")
}

#[test]
fn declaration_test() {
    run("let x = 42
    println(x)
    println(add(x, 10))
    let y = sub(x, 22)
    println(y)
    y = sub(y, y)
    println(y)")
}

#[test]
fn fn_dec() {
    run(r#"fn hello_world {
        println("Hello, World!")
    }
    
    hello_world()
    
    fn print_square {
        println(mul(args[0], args[0]))
    }
    
    print_square(10)
    print_square(10)
    print_square(10)
    print_square(10)
    print_square(10)
    print_square(10)"#)
}

#[test]
fn fn_in_loop() {
    run("let x = 0
    while neq(x, 10) {
        fn hello {
            println(\"Hello\", x)
        }
    
        hello() 
        x = add(x, 1)
    }")
}

#[test]
fn functions_test() {
    run(r#"println(add(3, 4), sub(5, 3.5), "Hello World")
    println("The answer", 
            "to", 
            "everything:",
            42,
            "",
            "Goodbye")
            
    print("The answer ", 
            "to ", 
            "everything: ",
            42,
            "\n",
            "Goodbye")
    "#)
}

#[test]
fn if_test() {
    run(r#"if and(true, false) {
        //will not be printed
        println("and")
    }
    
    if or(true, false) {
        println("or")
    }
    
    if eq(10, 10) {
        println("eq")
    }
    
    if not(or(true, false)) {
        println("nor")
    }
    
    if not(not(or(true, false))) {
        println("not nor")
    }
    "#)
}

#[test]
fn if_else() {
    run(r#"if not(eq(10, 10)) {
        println("first if branch")
    } else if neq(10, 10) {
        println("first else if")
    } else {
        //should execute
        println("first fall through")
    }
    
    if not(eq(10, 10)) {
        println("second if branch")
    } else if eq(10, 10) {
        //should execute
        println("second else if")
    }
    
    
    if eq(10, 10) {
        println("third if branch")
    } else if eq(10, 10) {
        println("third else if")
    }
    "#)
}

#[test]
fn pow_dec() {
    run("fn pow {
        let i = args[1]
    
        let res = 1
        while neq(i, 0){
            res = mul(res, args[0])
            i = sub(i, 1)
        }
    
        ret res
    }
    
    println(pow(2, 10))")
}

#[test]
fn closure_test() {
    run(r#"
    let fun = fn {
        println("Hello")
    }
    fun()
    fun = fn {
        print("Goodbye ", args[0], "\n")
    }
    fun("Ferris")"#)
}
