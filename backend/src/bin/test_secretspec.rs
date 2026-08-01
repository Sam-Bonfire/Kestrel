fn main() {
    println!("Hello");
    if let Ok(mut spec) = secretspec::Secrets::load() {
        if let Ok(resolved) = spec.resolve() {
            println!("{:?}", resolved.scope);
            // let's try to print the secrets
            // println!("{:?}", resolved.secrets);
        }
    }
}
