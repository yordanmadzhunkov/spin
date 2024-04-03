/*use spin_sdk::http::{IntoResponse, Response};
use spin_sdk::http_component;
use spin_sdk::llm;

/// A simple Spin HTTP component.
#[http_component]
fn hello_world(_req: http::Request<()>) -> anyhow::Result<impl IntoResponse> {
    Ok(Response::new(200, "Hello, world!"))
}
*/
use spin_sdk::{http::{IntoResponse, Request, Response}, http_component, llm};

// ttps://developer.fermyon.com/spin/v2/ai-sentiment-analysis-api-tutorial
// mkdir -p .spin/ai-models
// cd .spin/ai-models
// wget https://huggingface.co/TheBloke/Llama-2-13B-chat-GGML/resolve/a17885f653039bd07ed0f8ff4ecc373abf5425fd/llama-2-13b-chat.ggmlv3.q3_K_L.bin
// mv llama-2-13b-chat.ggmlv3.q3_K_L.bin llama2-chat

//Make sure it looks like this $ tree .spin
/*
.spin
└── ai-models
    └── llama2-chat
*/
// Next steps: 

// TODO WASI-NN, WASI GPU ??
// WASM NN, 
#[http_component]
fn hello_world(_req: Request) -> anyhow::Result<impl IntoResponse> {
   let model = llm::InferencingModel::Llama2Chat;
   let inference = llm::infer(model, "Can you tell me a joke about cats");
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!("{:?}", inference))
        .build())
}