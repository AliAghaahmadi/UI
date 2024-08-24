// main.rs

mod backend;

mod calculator;

use eframe::egui;
use calculator::CalculatorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(CalculatorApp::default()))
        }),
    )?;

    let input = "6*5 - 5515";

    // Tokenize the input string
    match tokenize::<f64>(input) {
        Ok(tokens) => {
            // Parse the tokens
            match parse(&tokens) {
                Ok(expr) => {
                    // Evaluate the expression
                    let mut interpreter = Interpreter::default();
                    // Example: No variables are set in this case
                    match interpreter.eval(&expr) {
                        Ok(result) => println!("Result: {}", result),
                        Err(error) => println!("Evaluation Error: {:?}", error),
                    }
                }
                Err(error) => {
                    println!("Parse Error: {:?}", error);
                }
            }
        }
        Err(error) => {
            println!("Tokenization Error: {:?}", error);
        }
    }
}
