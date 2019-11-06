use lambda_runtime::{handler_fn, LambdaCtx, run};
use serde_derive::{Deserialize, Serialize};
use simple_logger;

#[derive(Deserialize, Clone)]
struct CustomEvent {
    #[serde(rename = "firstName")]
    first_name: String,
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), fehler::Exception> {
    simple_logger::init()?;
    let func = handler_fn(func);
    run(func).await?;
    Ok(())
}

async fn func(event: String, _ctx: Option<LambdaCtx>) -> Result<String, Error> {
    let event:CustomEvent = serde_json::from_str(&event)?;

    Ok(serde_json::to_string(&CustomOutput{message:format!("The name is: {}", event.first_name)})?)
}
