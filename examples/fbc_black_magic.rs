use lake::FBC;
use lake::lake::memory::void::Void;
use std::thread;
use std::thread::JoinHandle;

fn main() {
    // Allocate a vector on the heap with capacity for 100 messages
    // Wrap it in `Void` to bypass all lifetimes and safety rules — because who needs those anyway?
    let mut messages: Void<Vec<String>> = FBC!(Vec::with_capacity(100));

    // Push some initial mystic messages
    messages.push("👁 I see everything...".to_string());
    messages.push("🔮 This buffer lives forever".to_string());

    // Log initial buffer contents
    println!("[Main] Initial: {:?}", *messages);

    // Spawn threads that will all share the same mutable buffer...
    // Yes, **mutable**, **shared**, **without locks**, and **no Sync**. We live dangerously.
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..3 {
        // Move the magical `Void<Vec<String>>` into each thread (because why not clone pointers like it's C)
        let handle: JoinHandle<()> = thread::spawn(move || {
            for j in 0..3 {
                // Mutate the shared vector concurrently with no synchronization
                // If this works — the gods of undefined behavior have blessed your machine.
                messages.push(format!("Thread {i} says hello {j}"));
                println!("[Thread {i}] {:?}", messages.last().unwrap());
            }
        });
        handles.push(handle);
    }

    // Wait for all chaos threads to complete
    for handle in handles {
        handle.join().unwrap(); // if it panics — you probably deserved it
    }

    // Main thread continues to push into the same shared vector
    messages.push("🎉 Final message from main thread".to_string());

    // Print final contents — assuming the vector wasn't corrupted, reallocated, or exploded mid-flight
    println!("[Main] Final buffer: {:?} ({})", *messages, messages.len());
}