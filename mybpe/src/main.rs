use mybpe::Tokenizer;

fn main() {
    println!("Testing mybpe tokenizer...");

    let tokenizer = Tokenizer::new();
    println!("Created tokenizer: merges={}, pattern length={}",
             tokenizer.merges.len(),
             tokenizer.pattern.len());

    let text = "Hello, world! Don't worry.";
    println!("\nEncoding text: {:?}", text);
    println!("Chunks:");
    let tokens = tokenizer.encode(text);
    println!("\nTokens: {:?}", tokens);
}
