1) I saw someone's code fail to compile because they 
were trying to send non-thread-safe data across threads. 
How does the Rust language allow for static (i.e. at compile time)
guarantees that specific data can be sent/shared acrosss threads?

Rust has the Send trait, which indicates that an owned value can be transferred from one thread to another,
and the Sync trait, which indicates that borrows of a value can be shared between different threads. These
two traits are what's known as marker traits, _i.e._ traits whose implementations are empty, used to indicate
certain properties that these implementors may hvbe.

2) Do you have to then implement the Send and Sync traits for 
every piece of data (i.e. a struct) you want to share and send across threads?

No, these traits are _auto_ traits, which means that the compiler automatically implements them when it is appropriate. For example, if a struct's attributes are all Send, the struct will automatically implement Send, even without being explicitly declared, unless it is opted out via a negative impl.

3) What types in the course have I seen that aren't Send? Give one example, 
and explain why that type isn't Send

The Rc struct (non-atomic reference counting pointer) is not Send. This is because its implementation, specifically regarding the incrementing and decrementing of its reference count, involves dereferencing raw pointers. The lack of Send and Sync provides the necessary guarantees for this operation to be safe.

4) What is the relationship between Send and Sync? Does this relate
to Rust's Ownership system somehow?

Generally, if a type `T` is Sync, a shared borrow of it, `&T`, will be Send, as it is safe to send a reference to another thread under the guarantee that no unsynchronised mutations can occur. 

The Send trait plays into the ownership system by indicating that the transfer of ownership of a value is safe, while the Sync trait does so by indicating that it is safe to share a borrow of a value between threads.

5) Are there any types that could be Send but NOT Sync? Is that even possible?
The Cell struct implements Send but not Sync, since the fact that you always operate on the owned value rather than references/borrows requires the guarantee that no other thread is operating on it simultaneously. It is fine to move it from one thread to another, as long as only one thread is ever working on it at any given time.

6) Could we implement Send ourselves using safe rust? why/why not?
No, because to implement Send manually involves bypassing the compiler's automatic checks. If the compiler thinks that a type is Send, it will already implement it for you. If you think that some type is thread-safe, despite the compiler not being able to verify it, you must use unsafe to manually implement Send and explain your reasoning in a _SAFETY_ comment.
