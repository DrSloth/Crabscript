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
fn closures() {
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

#[test]
fn scope_closures() {
    run(r#"
        fn gen_closure {
            const X = "This is const X"
            ret fn {
                println(X)
            }
        }

        let fun = gen_closure()
        fun()
        fun()
    "#)
}

#[test]
fn state_closures() {
    run(r#"
        fn counter {
            let x = 0
            ret fn {
                x = add(x, args[0])
                println(x)
            }
        }

        let cnt = counter()
        cnt(2)
        cnt(3)
        cnt(5)
    "#)
}

#[test]
fn state_closures2() {
    run(r#"
    let x = 2

    fn addo {
        x = add(1, x)
    }
    
    println(x)
    addo()
    println(x)
    addo()
    println(x)
    addo()
    println(x)
    "#)
}

#[test]
#[should_panic]
pub fn scopes_if() {
    run("
    if true {
        let x = 10
    }
    
    println(x)")
}

#[test]
#[should_panic]
pub fn scopes_while() {
    run("
    let cnt = 0
    while neq(cnt, 10) {
        let x = 10
        cnt = add(1, cnt)
    }
    
    println(x)")
}

#[test]
pub fn cmp_arr() {
    run("
    let a = array(1, 2, 3)
    if neq(a, array(1, 2, 3)) {
        panic()
    }
    ");
}

#[test]
#[should_panic]
pub fn cmp_arr2() {
    run("
    let a = array(1, 2, 3)
    if eq(a, array(1, 2, 3)) {
        panic()
    }
    ");
}

#[test]
pub fn range() {
    run("
    let r = range(0, 3)
    assert(eq(r(), 0))
    assert(eq(r(), 1))
    assert(eq(r(), 2))
    assert(eq(r(), none))
    assert(eq(r(), none))
    ");
}

#[test]
pub fn range2() {
    run("
    let r = range(0, 2)
    assert(eq(r(), 0))
    assert(eq(r(), 1))

    let r2 = r

    assert(eq(r(), none))
    assert(eq(r2(), none))
    ");
}

#[test]
pub fn range_rewind() {
    run("
    let r = range(0, 2)
    assert(eq(r(), 0))
    assert(eq(r(), 1))

    let r2 = rewind(r)

    assert(eq(r2(), 0))
    assert(eq(r2(), 1))

    assert(eq(r(), none))
    assert(eq(r2(), none))
    ");
}

#[test]
pub fn range_reverse() {
    run("
    let r = range(0, 2)
    let r2 = reverse(r)
    assert(eq(r(), 0))
    assert(eq(r(), 1))

    assert(eq(r2(), 1))
    assert(eq(r2(), 0))

    assert(eq(r(), none))
    assert(eq(r2(), none))
    ");
}

#[test]
pub fn range_reverse2() {
    run("
    let r = range(0, 2)
    assert(eq(r(), 0))
    assert(eq(r(), 1))

    let r2 = reverse(r)
    
    assert(eq(r2(), 1))
    assert(eq(r2(), 0))

    assert(eq(r(), none))
    assert(eq(r2(), none))
    ");
}

#[test]
pub fn reverse_range() {
    run("
    let r = range(2, 0)
    assert(eq(r(), 2))
    assert(eq(r(), 1))

    assert(eq(r(), none))
    ");
}

#[test]
pub fn reverse_range_reverse() {
    run("
    let r = range(2, 0)
    let r2 = reverse(r)
    assert(eq(r(), 2))
    assert(eq(r(), 1))

    assert(eq(r2(), 1))
    assert(eq(r2(), 2))   

    assert(eq(r(), none))
    assert(eq(r2(), none))
    ");
}

#[test]
pub fn arr_iter() {
    run("
    let it = iter(array(0, 1, 2))
    assert(eq(it(), 0))
    assert(eq(it(), 1))
    assert(eq(it(), 2))
    assert(eq(it(), none))
    ");
}

#[test]
pub fn arr_iter_reverse() {
    run("
    let it = iter(array(0, 1, 2))
    assert(eq(it(), 0))
    it = reverse(it)
    assert(eq(it(), 2))
    assert(eq(it(), 1))
    assert(eq(it(), 0))
    assert(eq(it(), none))
    ");
}

/*#[test]
pub fn iter_test() {
    run("
    let a = array(1, 2, 3)
    let b = collect_arr(map(iter(a), fn {
        ret mul(args[0], 2)
    }))

    if neq(b, array(2, 4, 6)) {
        panic()
    }
    ");
}*/
