use lambda::{handler_fn, LambdaCtx};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), fehler::Exception> {
    let func = handler_fn(func);
    lambda::run(func).await?;
    Ok(())
}

async fn func(event: String, _: Option<LambdaCtx>) -> Result<String, Error> {
    Ok(event)
}
