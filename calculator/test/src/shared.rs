// shared.rs

pub fn evaluate_expression(input: &str) -> Result<String, String> {
    use calculator::{parse, tokenize, Interpreter};

    match tokenize::<f64>(input) {
        Ok(tokens) => {
            match parse(&tokens) {
                Ok(expr) => {
                    let mut interpreter = Interpreter::default();
                    match interpreter.eval(&expr) {
                        Ok(result) => Ok(result.to_string()),
                        Err(error) => Err(format!("Evaluation Error: {:?}", error)),
                    }
                }
                Err(error) => Err(format!("Parse Error: {:?}", error)),
            }
        }
        Err(error) => Err(format!("Tokenization Error: {:?}", error)),
    }
}
