use actix_files::Files;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::{Datelike, Utc};
use dotenv::dotenv;
use pulldown_cmark::{html::push_html, Options, Parser};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct BudgetForm {
    age: Option<String>,
    income: Option<String>,
    housing: Option<String>,
    transportation: Option<String>,
    groceries: Option<String>,
}

// Define the state structure
struct AppState {
    budgets: Mutex<HashMap<String, HashMap<String, f32>>>, // Example structure for storing budgets
    budget_summary: Mutex<String>,
}
impl AppState {
    pub fn new() -> Self {
        AppState {
            budgets: Mutex::new(HashMap::new()),
            budget_summary: Mutex::new(String::new()),
        }
    }
}
#[derive(Serialize, Deserialize)]
struct Transaction {
    amount: f64,
    date: String,
    description: String,
}
async fn budget2(data: web::Data<AppState>, form: web::Form<BudgetForm>) -> impl Responder {
    // Example of how to process form data and calculate dummy values for the progress bars
    let income_as_number = match form.income.as_deref() {
        Some("Below $20,000") => 20000,
        Some("$20,000 - $40,000") => 30000,
        Some("$40,000 - $60,000") => 50000,
        Some("$60,000 - $80,000") => 70000,
        Some("Above $80,000") => 90000,
        _ => 60000,
    };

    let housing_percentage = match form.housing.as_deref() {
        Some("Owner - Mortgage") => 35,
        Some("Owner - No Mortgage") => 25,
        Some("Renter") => 45,
        Some("Living with Family") => 10,
        _ => 20,
    };

    let transportation_percentage = match form.transportation.as_deref() {
        Some("Public Transport") => 8,
        Some("Car - Loan") => 12,
        Some("Car - Owned") => 10,
        Some("Car - Leased") => 15,
        _ => 25,
    };

    let groceries_percentage = match form.groceries.as_deref() {
        Some("Single") => 10,
        Some("Couple") => 20,
        Some("Family") => 40,
        _ => 30,
    };

    let utilities_percentage = match form.groceries.as_deref() {
        Some("Single") => 5,
        Some("Couple") => 8,
        Some("Family") => 10,
        _ => 30,
    };
    let savings_investments_percentage = 100
        - (housing_percentage
            + transportation_percentage
            + groceries_percentage
            + utilities_percentage);

    let budget_summary = format!(
        "Income: {}, Housing: {}, Transportation: {}, Groceries: {}, Utilities: {}, Savings: {}",
        income_as_number,
        income_as_number * housing_percentage / 100,
        income_as_number * transportation_percentage / 100,
        income_as_number * groceries_percentage / 100,
        income_as_number * utilities_percentage / 100,
        income_as_number * savings_investments_percentage / 100
    );

    println!("Budget Summary: {}", budget_summary);
    {
        // write budgetstring to AppState
        let mut budget_data = data.budget_summary.lock().unwrap();
        *budget_data = budget_summary;
    }
    println!("Budget Summary: {}", data.budget_summary.lock().unwrap());

    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BudgetWrite</title>
    <link href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-light bg-light">
        <a href="/" class="navbar-brand">Let's Get Budgeting</a>
    </nav>
    <div class="container text-center">
        <h2>Budget Progress, income {income}</h2>
        <div class="row align-items-center mb-2">
            <div class="col-3 text-left">Housing</div>
            <div class="col-9">
                <div class="progress">
                    <div class="progress-bar bg-info" role="progressbar" style="width: {housing_percentage}%" aria-valuenow="{housing_percentage}" aria-valuemin="0" aria-valuemax="100">${housing_number}</div>
                </div>
            </div>
        </div>

        <div class="row align-items-center mb-2">
            <div class="col-3 text-left">Transportation</div>
            <div class="col-9">
                <div class="progress">
                    <div class="progress-bar bg-success" role="progressbar" style="width: {transportation_percentage}%" aria-valuenow="{transportation_percentage}" aria-valuemin="0" aria-valuemax="100">${transportation_number}</div>
                </div>
            </div>
        </div>

        <div class="row align-items-center mb-2">
            <div class="col-3 text-left">Groceries</div>
            <div class="col-9">
                <div class="progress">
                    <div class="progress-bar bg-warning" role="progressbar" style="width: {groceries_percentage}%" aria-valuenow="{groceries_percentage}" aria-valuemin="0" aria-valuemax="100">${groceries_number}</div>
                </div>
            </div>
        </div>

        <div class="row align-items-center mb-2">
            <div class="col-3 text-left">Utilities</div>
            <div class="col-9">
                <div class="progress">
                    <div class="progress-bar bg-danger" role="progressbar" style="width: {utilities_percentage}%" aria-valuenow="{utilities_percentage}" aria-valuemin="0" aria-valuemax="100">${utilities_number}</div>
                </div>
            </div>
        </div>

        <div class="row align-items-center mb-2">
            <div class="col-3 text-left">Savings & Investments</div>
            <div class="col-9">
                <div class="progress">
                    <div class="progress-bar bg-primary" role="progressbar" style="width: {savings_investments_percentage}%" aria-valuenow="{savings_investments_percentage}" aria-valuemin="0" aria-valuemax="100">${savings_number}</div>
                </div>
            </div>
        </div>
    </div>
      <div class="container text-center">
        <a href="/check" class="btn btn-primary">Check</a>
    </div>
</body>
</html>"#,
        housing_percentage = housing_percentage,
        housing_number = income_as_number * housing_percentage / 100,
        transportation_percentage = transportation_percentage,
        transportation_number = income_as_number * transportation_percentage / 100,
        groceries_percentage = groceries_percentage,
        groceries_number = income_as_number * groceries_percentage / 100,
        utilities_percentage = utilities_percentage,
        utilities_number = income_as_number * utilities_percentage / 100,
        savings_investments_percentage = savings_investments_percentage,
        savings_number = income_as_number * savings_investments_percentage / 100,
        income = income_as_number
    );

    // add all the numbers together in a string and write it to the AppState
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

pub async fn check(data: web::Data<AppState>) -> impl Responder {
    let budget_data = {
        let data = data.budget_summary.lock().unwrap();
        data.clone()
    };
    // Dummy list of transactions
    let transactions = vec![
        Transaction {
            amount: 100.0,
            date: "2020-10-01".to_string(),
            description: "Salary".to_string(),
        },
        Transaction {
            amount: -50.0,
            date: "2020-10-02".to_string(),
            description: "Groceries".to_string(),
        },
        Transaction {
            amount: -20.0,
            date: "2020-10-03".to_string(),
            description: "Coffee".to_string(),
        },
        Transaction {
            amount: -10.0,
            date: "2020-10-04".to_string(),
            description: "Lunch".to_string(),
        },
        Transaction {
            amount: -30.0,
            date: "2020-10-05".to_string(),
            description: "Dinner".to_string(),
        },
    ];

    // add budget information

    let prompt = format!(
        "{} , this is my yearly budget and the following is my transactions for the day, assume today is indicative of my spending habits. take a harsh approach but in a way that has a goal of overall being coaching",
        budget_data
    );
    let combined = transactions
        .iter()
        .map(|t| format!("{}: ${} - {}", t.date, t.amount, t.description))
        .collect::<Vec<String>>()
        .join("\n");
    let message = format!("{}\n\n{}", prompt, combined);

    // Hit the OpenAI API with the message

    dotenv::dotenv().ok(); // Load environment variables from .env file
    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not found in .env file");

    let client = Client::new();

    let payload = json!({
        "contents": [{
            "parts": [{"text": message}]
        }]
    });

    // Use the API key directly in the URL
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request")
        .text()
        .await
        .expect("Failed to read response");

    // Parse JSON response and extract the text content
    let v: Value = match serde_json::from_str(&response) {
        Ok(val) => val,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to parse JSON response.")
        }
    };

    let markdown_text = v["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("");

    // Convert Markdown to HTML
    let mut html_output = String::new();
    let parser = Parser::new_ext(markdown_text, Options::all());
    push_html(&mut html_output, parser);
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Response Analysis</title>
    <link href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" rel="stylesheet">
    <style>
        .response-container {{
            margin-top: 20px;
            padding: 15px;
            background-color: #f8f9fa;
            border-left: 5px solid #007bff;
            font-family: Arial, sans-serif;
        }}
    </style>
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-light bg-light">
        <a href="/" class="navbar-brand">Let's Get Budgeting</a>
    </nav>
    <div class="container">
        <h1>AI Response</h1>
        <div class="response-container">
            <p>{}</p>
        </div>
    </div>
</body>
</html>"#,
        html_output
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

async fn new_landing() -> impl Responder {
    let body = r#"
    <!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to Our Service</title>
    <!-- Bootstrap CSS -->
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <div class="container">
            <a class="navbar-brand" href="\#"BdgtWrite</a>
            <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="\#navbarNav" aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                <span class="navbar-toggler-icon"></span>
            </button>
            <div class="collapse navbar-collapse" id="navbarNav">
                <ul class="navbar-nav">
                    <li class="nav-item">
                        <a class="nav-link active" aria-current="page" href="\#hero">Home</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="\#about">About</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href="\#services">Services</a>
                    </li>
                    <li class="nav-item">
                        <a class="nav-link" href=#contact">Contact</a>
                    </li>
                </ul>
            </div>
        </div>
    </nav>

    <section id="hero" class="bg-primary text-white text-center p-5">
        <div class="container">
            <h1>Welcome to BdgtWrite</h1>
            <p class="lead">Discover how we can transform your life.</p>
            <a href="\personal" class="btn btn-light btn-lg">Start your budget!</a>
        </div>
    </section>

   
    <div class="container py-5" id="services">
        <div class="row">
            <div class="col-lg-8 mx-auto">
                <h2>Our Services</h2>
                <p class="lead">Our personal finance app guides you through interactive questions to help you build a personalized budget. Discover how easy it is to manage your finances, track your spending, and achieve your financial goals.</p>
                <ul class="list-unstyled">
                <li>🔍 <strong>Budget Creation:</strong> Step-by-step guidance to create a budget that fits your lifestyle.</li>
                <li>📊 <strong>Spending Tracker:</strong> Real-time insights into your spending habits.</li>
                <li>💡 <strong>Financial Tips:</strong> Smart tips tailored to your financial situation to help you save more.</li>
                <li>🎯 <strong>Goal Setting:</strong> Set and achieve financial goals with actionable plans.</li>
                <li>🔗 <strong>Account Integration:</strong> Seamlessly connect and view all your accounts in one place.</li>
            </ul>
            <p class="mt-3">Start taking control of your financial future today with our user-friendly personal finance tool designed to make budgeting simple and effective.</p>
            </div>
        </div>
    </div>
 <div class="container py-5" id="about">
        <div class="row">
            <div class="col-lg-8 mx-auto">
                <h2>About Us</h2>
                <p class="lead">We are innovators in the digital landscape, dedicated to enhancing your personal finance through tailored solutions.</p>
            </div>
        </div>
    </div>

    <footer class="bg-dark text-white text-center p-3">
        <div class="container">
            <p>Copyright &copy; 2024 Your Brand. All rights reserved.</p>
        </div>
    </footer>

    <!-- Bootstrap JS Bundle with Popper -->
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/js/bootstrap.bundle.min.js"></script>
</body>
</html>"#;

    actix_web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

pub async fn budget() -> impl Responder {
    let body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>BudgetWrite</title>
    <link href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-light bg-light">
        <a href="/" class="navbar-brand">Let's Get Budgeting</a>
    </nav>
    <div class="container text-center">
        <h2>Budget Progress</h2>
        <div class="row align-items-center">
            <div class="col-4 text-right">Housing</div>
            <div class="col-8">
                <div class="progress mb-2">
                    <div class="progress-bar bg-info" role="progressbar" style="width: 20%" aria-valuenow="20" aria-valuemin="0" aria-valuemax="100">20%</div>
                </div>
            </div>

            <div class="col-4 text-right">Transportation</div>
            <div class="col-8">
                <div class="progress mb-2">
                    <div class="progress-bar bg-success" role="progressbar" style="width: 40%" aria-valuenow="40" aria-valuemin="0" aria-valuemax="100">40%</div>
                </div>
            </div>

            <div class="col-4 text-right">Groceries</div>
            <div class="col-8">
                <div class="progress mb-2">
                    <div class="progress-bar bg-warning" role="progressbar" style="width: 60%" aria-valuenow="60" aria-valuemin="0" aria-valuemax="100">60%</div>
                </div>
            </div>

            <div class="col-4 text-right">Utilities</div>
            <div class="col-8">
                <div class="progress mb-2">
                    <div class="progress-bar bg-danger" role="progressbar" style="width: 80%" aria-valuenow="80" aria-valuemin="0" aria-valuemax="100">80%</div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}

// Home page and form handler
async fn home(data: web::Data<AppState>) -> impl Responder {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Interactive Budget Form</title>
    <link href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css" rel="stylesheet">
</head>
<body>
    <nav class="navbar navbar-expand-lg navbar-light bg-light">
        <a href="/" class="navbar-brand">Home</a>
    </nav>
    <div class="container">
        <h1>Set Your Budget</h1>
        <p>Help us get to know you so we can set a budget that fits your lifestyle.</p>
        <form action="/set_budget" method="post">
            <div class="form-group">
                <label for="age">Age Group:</label>
                <select class="form-control" id="age" name="age">
                    <option>18-25</option>
                    <option>26-35</option>
                    <option>36-45</option>
                    <option>46-55</option>
                    <option>56+</option>
                </select>
            </div>
            <div class="form-group">
                <label for="income">Income Range:</label>
                <select class="form-control" id="income" name="income">
                    <option>Below $20,000</option>
                    <option>$20,000 - $40,000</option>
                    <option>$40,000 - $60,000</option>
                    <option>$60,000 - $80,000</option>
                    <option>Above $80,000</option>
                </select>
            </div>
            <div class="form-group">
                <label for="housing">Housing:</label>
                <select class="form-control" id="housing" name="housing">
                    <option>Owner - Mortgage</option>
                    <option>Owner - No Mortgage</option>
                    <option>Renter</option>
                    <option>Living with Family</option>
                </select>
            </div>
            <div class="form-group">
                <label for="transportation">Transportation:</label>
                <select class="form-control" id="transportation" name="transportation">
                    <option>Public Transport</option>
                    <option>Car - Loan</option>
                    <option>Car - Owned</option>
                    <option>Car - Leased</option>
                </select>
            </div>
            <div class="form-group">
                <label for="groceries">Groceries:</label>
                <select class="form-control" id="groceries" name="groceries">
                    <option>Single</option>
                    <option>Couple</option>
                    <option>Family</option>
                </select>
            </div>

            <button type="submit" class="btn btn-primary">Submit</button>
        </form>
    </div>
</body>
</html>"#
    );
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

// Budget setting and state update handler
async fn set_budget(
    form: web::Form<HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let demographics = form.into_inner();
    let age = demographics
        .get("age")
        .cloned()
        .unwrap_or("Unknown".to_string());

    let income = demographics
        .get("income")
        .cloned()
        .unwrap_or("Unknown".to_string());
    let budget_percentages = calculate_budget(&age, &income);
    let mut budgets = data.budgets.lock().unwrap(); // Lock the state
    let budget_targets = budgets.entry(age.clone()).or_insert_with(HashMap::new);
    budget_targets.insert(
        income.clone(),
        budget_percentages.get("Housing").unwrap().clone(),
    );
    println!("HashMap = {:?}", budget_percentages);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "Budget set for Age: {}, Income: {}, Set Budget: {},
            Housing: {}, Transportation: {}, Groceries: {}, Utilities: {}",
            age,
            income,
            budget_targets.get(&income).unwrap(),
            budget_percentages.get("Housing").unwrap(),
            budget_percentages.get("Transportation").unwrap(),
            budget_percentages.get("Groceries").unwrap(),
            budget_percentages.get("Utilities").unwrap(),
        ))
}

fn calculate_budget(age: &str, income: &str) -> HashMap<String, f32> {
    let mut budget_percentages = HashMap::new();
    let base_percentage = match income {
        "Below $20,000" => 50.0,
        "$20,000 - $40,000" => 60.0,
        "$40,000 - $60,000" => 70.0,
        "$60,000 - $80,000" => 80.0,
        "Above $80,000" => 90.0,
        _ => 60.0, // Default case if income doesn't match any of the categories
    };

    // Assuming base_percentage is the starting point, adjust based on categories
    budget_percentages.insert("Housing".to_string(), base_percentage * 0.4); // 40% of base
    budget_percentages.insert("Transportation".to_string(), base_percentage * 0.2); // 20% of base
    budget_percentages.insert("Groceries".to_string(), base_percentage * 0.2); // 20% of base
    budget_percentages.insert("Utilities".to_string(), base_percentage * 0.2); // 20% of base

    budget_percentages
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_data = web::Data::new(AppState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/", web::get().to(new_landing))
            .route("/budget", web::get().to(budget2))
            .route("/check", web::get().to(check))
            .route("/personal", web::get().to(home)) // Serve home page with form
            .route("/set_budget", web::post().to(budget2))
            .service(actix_files::Files::new("/static", "static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
