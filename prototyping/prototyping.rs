// Welcome!
//
// This file contains snippets of prototype implementations of the asyncronous node structure,
// among other prototypes created during the development process.
//
// Whilst the age of this repository suggests that "rome was built in a day",
// this file should hopefully showcase some of the pain that was crawled, so that MechSync could run.
//
// This file is therefore not expected to run, and the code is not used within the final artifact.

use std::error::Error;
use std::io::stdin;
use std::sync::Arc;
use std::time::Duration;

use async_recursion::async_recursion;
use futures::future::{BoxFuture, FutureExt};
use may::coroutine::sleep;
use may::go;
use may::sync::{Mutex, RwLock};
use midir::{ConnectError, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use midir::os::unix::{VirtualInput, VirtualOutput};

// included during experiments with tokio async
use tokio::runtime::Handle;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

use crate::Node::{Func, Input, Output};

type Data = [u8; 3];

fn main() {
    match run() {
        Ok(_) => {}
        Err(_) => {}
    }
}

// Here we see a horrifically contrived attempt to build dynamic async functions.
// This code is hideous and unreadable, no wonder I shifted to coroutines!
type BoxedFunc<'a, T, U> = Box<dyn Fn(T) -> BoxFuture<'a, U> + Send + Sync>;

trait AsyncBox<T, U> {
    fn boxed(&'static self) -> BoxedFunc<'static, T, U>;
}

// Imagine being the poor honours student next year who had to try and comprehend what this does!
impl<T, U, F, Fut> AsyncBox<T, U> for F where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = U> + Send + 'static {
    fn boxed(&'static self) -> BoxedFunc<'static, T, U> {
        Box::new(move | t | {
            self(t).boxed()
        })
    }
}

// Originally, trial systems implemented nodes using Enums. This however wasn't very flexible.
// To get any actual dynamic functionality, dyn Fn had to be used, which within the async prototypes was a mess.
enum Node {
    Input{ children: RwLock<Vec<Arc<Node>>> },
    Output{ output: Arc<Mutex<MidiOutputConnection>> },
    Func{ func: Box<dyn Fn(Data) -> Data + Send + Sync>, children: Vec<Arc<Node>> },
}

impl Node {
    fn func(func: Box<dyn Fn(Data) -> Data + Send + Sync>) -> Node {
        Func {func, children: vec![]}
    }

    fn output(output: Arc<Mutex<MidiOutputConnection>>) -> Node {
        Output { output }
    }

    fn input(input: MidiInput, name: &str) -> Result<(&'static Node, MidiInputConnection<()>), ConnectError<MidiInput>> {
        let node = Box::leak(Box::new(Input { children: RwLock::new(vec![]) }));
        let handle = build_virtual_input_node(input, name, node)?;
        Ok((node, handle))
    }

    fn call(&self, data: Data) -> () {
        // of course, to do anything, we have to match on self, which gets... very... very... verbose...
        match self {
            Input { children} => {
                let borrow_guard = children.read().expect("explode");
                for child in borrow_guard.iter() {
                    child.call(data);
                }
            }
            Output { output } => {
                println!("output: {:?}", data);
                output.lock().expect("lock poisoned!").send(data.as_slice()).expect("Failed to send midi data!");
            }
            Func { func, children } => {
                let res = func(data);
                for child in children {
                    child.call(res);
                }
            }
        }
    }

    fn add_input_child(&self, child: Arc<Node>) {
        match self {
            Input { children } => {
                children.write().unwrap().push(child);
            }
            _ => {}
        }
    }

    fn add_func_child(&mut self, child: Arc<Node>) {
        match self {
            Func { children, .. } => {
                children.push(child);
            }
            _ => {}
        }
    }
}


// Here we see the graveyard of 𝐨𝐡 𝐧𝐨 𝐡𝐨𝐰 𝐝𝐨 𝐈 𝐮𝐬𝐞 𝐚𝐬𝐲𝐧𝐜 𝐰𝐢𝐭𝐡 𝐭𝐫𝐚𝐢𝐭𝐬 𝐢𝐧 𝐫𝐮𝐬𝐭, 𝐬𝐞𝐧𝐝 𝐡𝐞𝐥𝐩

trait Node<T, U> {
    async fn call(&self, data: T) -> U;

    fn chain<'a, N, V>(&'a self, other: &'a N) -> ChainNode<Self, N, T, U, V> where N: Node<U, V>, Self: Sized {
        return ChainNode { a: self, b: other, p_t: Default::default(), p_u: Default::default(), p_v: Default::default() };
    }
}

struct IntNode {}

impl Node<i32, i32> for IntNode {
    async fn call(&self, data: i32) -> i32 {
        println!("{}", data);
        data
    }
}

struct OutNode {
    output: Arc<Mutex<MidiOutputConnection>>
}

impl Node<[u8; 3], ()> for OutNode {
    async fn call(&self, data: [u8; 3]) -> () {
        self.output.lock().await.send(data.as_slice()).expect("TODO: panic message");
    }
}



// Arguably, one of the funniest types I have ever written.
// Someone obviously was losing their mind here (me)

struct ChainNode<'a, N1, N2, T, U, V> where
    N1: Node<T, U>,
    N2: Node<U, V>{
    a: &'a N1,
    b: &'a N2,
    p_t: PhantomData<T>,
    p_u: PhantomData<U>,
    p_v: PhantomData<V>
}

impl<N1, N2, T, U, V> Node<T, V> for ChainNode<'_, N1, N2, T, U, V> where
    N1: Node<T, U>,
    N2: Node<U, V> {
    async fn call(&self, data: T) -> V {
        self.b.call(self.a.call(data).await).await
    }
}

async fn test_wrapper<F, Fut>(next: F, n: i32) where
    F: Fn(i32) -> Fut,
    Fut: Future<Output = ()> {
    next(n).await;
}

fn compose<F, F2, Fut, Fut2>(func: F, next: F2) -> impl Fn(i32) where
    F: Fn(F2, i32) -> Fut,
    Fut: Future<Output = ()>,
    F2: Fn(i32) -> Fut2,
    Fut2: Future<Output = ()> {
    return async move |n: i32| func(next, n).await;
}

trait Node {
    fn call(&self, data: [u8; 3]) -> impl Future<Output = ()> + Send;
}

struct OutNode {
    out: Arc<Mutex<MidiOutputConnection>>
}

impl OutNode {
    fn new(out: Arc<Mutex<MidiOutputConnection>>) -> Self {
        Self {
            out,
        }
    }
}

impl Node for OutNode {
    async fn call(&self, data: [u8; 3]) -> () {
        self.out.lock().await.send(data.as_slice()).expect("Sending failed");
    }
}

// This function has to be the most painful function I have ever had the displeasure of rewriting
// notice how node is 'static? Yeah, we have to 𝗹𝗲𝗮𝗸 𝗺𝗲𝗺𝗼𝗿𝘆 𝗼𝗻 𝗽𝘂𝗿𝗽𝗼𝘀𝗲 to make that work. Horrible.
//
// Now, I'm not saying that unsafe pointer magic used in the new implementation is any nicer, but
// at least we create them at runtime without causing a memory leak. Glorious.
fn build_virtual_input_node(
    input: MidiInput,
    port_name: &str,
    node: &'static Node
) -> Result<MidiInputConnection<()>, ConnectError<MidiInput>> {
    input.create_virtual(port_name, move |_ts, data, ()| {
        let mut data_cpy = [0u8; 3];
        for (x, y) in data.iter().zip(data_cpy.iter_mut()) {
            *y = *x;
        }

        // May goroutines single-handedly saved this project.
        go!(move || {
            node.call(data_cpy);
        });
    }, ())
}

// Look at the absolute insanity required to make bindable async functions work for Midir!
fn build_virtual_input<F, Fut>(
    runtime: &mut Runtime,
    input: MidiInput,
    port_name: &str,
    functor: F
) -> Result<MidiInputConnection<()>, ConnectError<MidiInput>> where
    F: Fn([u8; 3]) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static {
    let handle = runtime.handle().clone(); //sendable_chungus.clone() - https://bitbashing.io/async-rust.html
    let functor = functor;
    input.create_virtual(port_name, move |_ts, data, _| {
        let data: [u8; 3] = data.try_into().unwrap(); // yeah, this line panics if a 2-byte instruction is encountered, good job me :)
        handle.spawn(functor(data));
    }, ())
}

// with May, no more coloured functions! What's an `impl Future<Output = ()>` again?
fn sleep_(data: Data) -> Data {
    println!("Midi!");
    sleep(Duration::from_millis(500));
    data
}

// all this code does it takes some input, delays it by 500ms, then returns it
fn run() -> Result<(), Box<dyn Error>> {
    let input = MidiInput::new("virtual_input_01")?;
    let output = MidiOutput::new("virtual_output_01")?;
    let virtual_output = output.create_virtual("v_out")?;

    let out = Arc::new(Mutex::new(virtual_output));

    let (node, handle) = Node::input(input, "v_in")?;

    let mut delay = Node::func(Box::new(sleep_));
    let outnode = Arc::new(Node::output(out));
    delay.add_func_child(outnode);
    node.add_input_child(Arc::new(delay));

    println!("waiting for connection to close");
    let mut input = String::new();
    stdin().read_line(&mut input)?;

    Ok(())
}
