use anyhow::Result;
use cowlang::Program;

#[tokio::main]
async fn main() -> Result<()> {
    let program = Program::parse(include_str!("../../cowlang-samples/hello-world-loops.txt"));
    cowlang_viz::vizualize(program).await
}
